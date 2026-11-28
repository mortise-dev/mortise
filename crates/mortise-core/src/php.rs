//! The only module that touches the PHP parser.
//!
//! The parser's API changes between minor versions, so nothing outside this
//! file names one of its types. What leaves here is owned, plain data.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use mago_allocator::LocalArena;
use mago_database::file::{File, FileId, FileType};
use mago_names::ResolvedNames;
use mago_names::resolver::NameResolver;
use mago_span::HasSpan;
use mago_syntax::cst::{
    Argument, ArgumentList, ArrayElement, Attribute, AttributeList, Binary, BinaryOperator, Call,
    Class, ClassConstantAccess, ClassLikeMemberSelector, Enum, Expression, Extends, FunctionCall,
    Hint, Identifier, Implements, Instantiation, Interface, Literal, LocalIdentifier, Method,
    MethodCall, Modifier, PartialArgument, PropertyConcreteItem, Sequence, Statement,
    StaticMethodCall, StaticMethodPartialApplication, StaticPropertyAccess, TokenSeparatedSequence,
    Trait, TraitUse, TriviaKind, Use, UseItems,
};
use mago_syntax::parser::{parse_file_content, parse_file_with_settings};
use mago_syntax::settings::ParserSettings;
use mago_syntax::walker::MutWalker;

/// A value in a PHP config file, as far as it can be known without running PHP.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ConfigValue {
    Str(String),
    Bool(bool),
    Int(i64),
    Null,
    /// `base_path('...')`: a path under the app folder. An empty argument is the app folder.
    BasePath(String),
    Array(Vec<(Option<ConfigValue>, ConfigValue)>),
    /// Anything that needs PHP to evaluate: `env()`, concatenation, a constant, a closure.
    Unknown,
}

impl ConfigValue {
    /// The value under a string key of an array.
    pub(crate) fn get(&self, key: &str) -> Option<&ConfigValue> {
        let ConfigValue::Array(items) = self else {
            return None;
        };
        // PHP keeps the last of two equal keys.
        items.iter().rev().find_map(|(k, v)| match k {
            Some(ConfigValue::Str(k)) if k == key => Some(v),
            _ => None,
        })
    }

    /// The value at a path of string keys.
    pub(crate) fn at(&self, path: &[&str]) -> Option<&ConfigValue> {
        path.iter().try_fold(self, |value, key| value.get(key))
    }

    /// The value at a path of string keys, or whether it is missing or cannot be
    /// known without PHP: a part of the path is not an array, or an array that lacks
    /// the key has a spread or a computed key that could hold it.
    pub(crate) fn lookup(&self, path: &[&str]) -> Lookup<'_> {
        let mut value = self;
        for key in path {
            let ConfigValue::Array(items) = value else {
                return Lookup::Unresolved;
            };
            match value.get(key) {
                Some(next) => value = next,
                None => {
                    let open = items.iter().any(|(k, v)| match k {
                        Some(k) => *k == ConfigValue::Unknown,
                        None => *v == ConfigValue::Unknown,
                    });
                    return match open {
                        true => Lookup::Unresolved,
                        false => Lookup::Missing,
                    };
                }
            }
        }
        Lookup::Found(value)
    }
}

/// What a path of keys gives in a config value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Lookup<'a> {
    Found(&'a ConfigValue),
    /// No such key: the package default holds.
    Missing,
    /// Only PHP can tell.
    Unresolved,
}

/// Reads the array a PHP config file returns.
///
/// Returns `None` when the file has no `return` statement at its top level.
pub(crate) fn read_config(source: &[u8]) -> Option<ConfigValue> {
    let arena = LocalArena::new();
    let program = parse_file_content(&arena, FileId::zero(), source);
    program
        .statements
        .iter()
        .find_map(|statement| match statement {
            Statement::Return(ret) => ret.value.map(config_value),
            _ => None,
        })
}

fn config_value(expression: &Expression<'_>) -> ConfigValue {
    match expression {
        Expression::Parenthesized(inner) => config_value(inner.expression),
        Expression::Literal(literal) => match literal {
            Literal::String(string) => match string.value {
                Some(value) => ConfigValue::Str(lossy(value)),
                None => ConfigValue::Unknown,
            },
            Literal::Integer(integer) => integer
                .value
                .and_then(|value| i64::try_from(value).ok())
                .map_or(ConfigValue::Unknown, ConfigValue::Int),
            Literal::True(_) => ConfigValue::Bool(true),
            Literal::False(_) => ConfigValue::Bool(false),
            Literal::Null(_) => ConfigValue::Null,
            Literal::Float(_) => ConfigValue::Unknown,
        },
        Expression::Array(array) => array_value(&array.elements),
        Expression::LegacyArray(array) => array_value(&array.elements),
        Expression::Call(Call::Function(call)) => {
            let Expression::Identifier(name) = call.function else {
                return ConfigValue::Unknown;
            };
            if !name.last_segment().eq_ignore_ascii_case(b"base_path") {
                return ConfigValue::Unknown;
            }
            let mut arguments = call.argument_list.arguments.iter();
            match (arguments.next(), arguments.next()) {
                (None, _) => ConfigValue::BasePath(String::new()),
                (Some(Argument::Positional(argument)), None) if argument.ellipsis.is_none() => {
                    match config_value(argument.value) {
                        ConfigValue::Str(path) => ConfigValue::BasePath(path),
                        _ => ConfigValue::Unknown,
                    }
                }
                _ => ConfigValue::Unknown,
            }
        }
        _ => ConfigValue::Unknown,
    }
}

fn array_value(elements: &TokenSeparatedSequence<'_, ArrayElement<'_>>) -> ConfigValue {
    ConfigValue::Array(
        elements
            .iter()
            .filter_map(|element| match element {
                ArrayElement::KeyValue(pair) => {
                    Some((Some(config_value(pair.key)), config_value(pair.value)))
                }
                ArrayElement::Value(item) => Some((None, config_value(item.value))),
                ArrayElement::Variadic(_) => Some((None, ConfigValue::Unknown)),
                ArrayElement::Missing(_) => None,
            })
            .collect(),
    )
}

fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Counts the tests in a PHP test file: PHPUnit methods whose name starts with
/// `test`, or that carry the `Test` attribute or a `@test` docblock, and Pest's
/// `it(...)` and `test(...)` calls.
pub(crate) fn count_tests(source: &[u8]) -> u32 {
    let arena = LocalArena::new();
    let program = parse_file_content(&arena, FileId::zero(), source);
    let mut counter = TestCounter::default();
    counter.walk_program(program, &mut ());

    // A docblock with @test marks the method that follows it.
    let mut tests = counter.calls
        + counter
            .methods
            .iter()
            .filter(|(_, is_test)| *is_test)
            .count() as u32;
    for trivia in program.trivia.iter() {
        if trivia.kind != TriviaKind::DocBlockComment || !has_test_tag(trivia.value) {
            continue;
        }
        let end = trivia.span.end.offset;
        let next = counter
            .methods
            .iter()
            .filter(|(start, _)| *start >= end)
            .min_by_key(|(start, _)| *start);
        if let Some((_, false)) = next {
            tests += 1;
        }
    }
    tests
}

fn has_test_tag(docblock: &[u8]) -> bool {
    docblock.windows(5).enumerate().any(|(at, window)| {
        window == b"@test" && !docblock.get(at + 5).is_some_and(u8::is_ascii_alphanumeric)
    })
}

#[derive(Default)]
struct TestCounter {
    /// Where each method starts, and whether its name or an attribute makes it a test.
    methods: Vec<(u32, bool)>,
    /// Pest's `it` and `test` calls.
    calls: u32,
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for TestCounter {
    fn walk_in_method(&mut self, method: &'ast Method<'arena>, _: &mut ()) {
        // PHPUnit runs public methods whose name starts with "test".
        let hidden = method
            .modifiers
            .iter()
            .any(|modifier| matches!(modifier, Modifier::Private(_) | Modifier::Protected(_)));
        let name = method.name.value;
        let by_name = !hidden && name.len() >= 4 && name[..4].eq_ignore_ascii_case(b"test");
        let by_attribute = method
            .attribute_lists
            .iter()
            .flat_map(|list| list.attributes.iter())
            .any(|attribute| attribute.name.last_segment().eq_ignore_ascii_case(b"Test"));
        self.methods
            .push((method.function.span.start.offset, by_name || by_attribute));
    }

    fn walk_in_function_call(&mut self, call: &'ast FunctionCall<'arena>, _: &mut ()) {
        if let Expression::Identifier(name) = call.function
            && (name.value() == b"it" || name.value() == b"test")
        {
            self.calls += 1;
        }
    }
}

/// The class of Laravel's `Schema` facade.
const SCHEMA: &[u8] = b"Illuminate\\Support\\Facades\\Schema";

/// The tables a migration creates: the table of each `Schema::create(...)` and
/// `Schema::connection(...)->create(...)` in its `up` method, when it is a string
/// literal, as the first argument or the named argument `table`. `Schema` is the
/// facade under any name the file's `use` gives it, its full name, or the plain
/// `Schema` that Laravel's alias gives. In order, each once.
pub(crate) fn created_tables(source: &str) -> Vec<String> {
    let arena = LocalArena::new();
    let program = parse_file_content(&arena, FileId::zero(), source.as_bytes());
    let names = NameResolver::new(&arena).resolve(program);
    let mut reader = TableReader {
        names: &names,
        methods: Vec::new(),
        tables: Vec::new(),
    };
    reader.walk_program(program, &mut ());
    reader.tables
}

struct TableReader<'a, 'arena> {
    names: &'a ResolvedNames<'arena>,
    /// For each method the walk is in, whether it is `up`.
    methods: Vec<bool>,
    tables: Vec<String>,
}

impl TableReader<'_, '_> {
    fn in_up(&self) -> bool {
        self.methods.last().copied().unwrap_or(false)
    }

    /// The table of a `create` call with these arguments.
    fn create(&mut self, method: &ClassLikeMemberSelector<'_>, arguments: &ArgumentList<'_>) {
        if !self.in_up() || !is_named(method, b"create") {
            return;
        }
        let table = arguments
            .arguments
            .iter()
            .enumerate()
            .find_map(|(at, argument)| match argument {
                Argument::Positional(argument) if at == 0 => Some(argument.value),
                Argument::Named(argument) if argument.name.value == b"table" => {
                    Some(argument.value)
                }
                _ => None,
            })
            .and_then(string_of);
        if let Some(table) = table
            && !self.tables.contains(&table)
        {
            self.tables.push(table);
        }
    }

    /// Whether an expression is the `Schema` facade: its full name once resolved
    /// through the file's `use`, or the global `Schema` of Laravel's alias. A class the
    /// file imports as `Schema` resolves to its own name, so it is not the facade.
    fn is_schema(&self, class: &Expression<'_>) -> bool {
        let Expression::Identifier(name) = class else {
            return false;
        };
        let resolved = self.names.resolve(&name.span()).unwrap_or(name.value());
        let resolved = resolved.strip_prefix(b"\\").unwrap_or(resolved);
        resolved.eq_ignore_ascii_case(SCHEMA) || resolved.eq_ignore_ascii_case(b"Schema")
    }
}

/// Whether a method selector is the plain name `name`. PHP method names ignore case.
fn is_named(method: &ClassLikeMemberSelector<'_>, name: &[u8]) -> bool {
    matches!(method, ClassLikeMemberSelector::Identifier(identifier) if identifier.value.eq_ignore_ascii_case(name))
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for TableReader<'_, 'arena> {
    fn walk_in_method(&mut self, method: &'ast Method<'arena>, _: &mut ()) {
        self.methods
            .push(method.name.value.eq_ignore_ascii_case(b"up"));
    }

    fn walk_out_method(&mut self, _: &'ast Method<'arena>, _: &mut ()) {
        self.methods.pop();
    }

    // `Schema::create('orders', ...)`.
    fn walk_in_static_method_call(&mut self, call: &'ast StaticMethodCall<'arena>, _: &mut ()) {
        if self.is_schema(call.class) {
            self.create(&call.method, &call.argument_list);
        }
    }

    // `Schema::connection('reports')->create('orders', ...)`.
    fn walk_in_method_call(&mut self, call: &'ast MethodCall<'arena>, _: &mut ()) {
        if let Expression::Call(Call::StaticMethod(connection)) = call.object
            && self.is_schema(connection.class)
            && is_named(&connection.method, b"connection")
        {
            self.create(&call.method, &call.argument_list);
        }
    }
}

/// A class, interface, trait or enum that a PHP file names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClassRef {
    /// The full name, without a leading backslash: `Modules\Billing\Models\Invoice`.
    pub(crate) class: String,
    /// The first line of the file that names it. Lines start at 1.
    pub(crate) line: u32,
}

/// What kind of thing a PHP file declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeclaredKind {
    Class,
    Interface,
    Trait,
    Enum,
}

/// A class, interface, trait or enum that a PHP file declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Declared {
    pub(crate) class: String,
    pub(crate) kind: DeclaredKind,
    /// The full names of the attributes on it, such as `Illuminate\Console\Attributes\Signature`.
    pub(crate) attributes: Vec<String>,
    /// For a console command: its name, from its signature. `billing:send-invoices`
    pub(crate) command: Option<String>,
    /// The text of a `$name` property. An older command has its name there, but so
    /// can any other class: it counts only for a class in the commands folder.
    pub(crate) name_property: Option<String>,
}

/// What a PHP file declares and what it names, with names resolved the way PHP
/// resolves them: through `use`, aliases and the namespace of the file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Analysis {
    pub(crate) declares: Vec<Declared>,
    /// Every other class the file names, once each, in the order of their first line.
    pub(crate) refs: Vec<ClassRef>,
}

/// Reads what a PHP file declares and names. A file with syntax errors gives what
/// could be read around them; it never fails.
pub(crate) fn analyze(source: &[u8]) -> Analysis {
    let arena = LocalArena::new();
    let file = File::new(
        Cow::Borrowed(b"file.php"),
        FileType::Host,
        None,
        Cow::Owned(source.to_vec()),
    );
    let settings = ParserSettings {
        recover_incomplete: true,
        ..ParserSettings::default()
    };
    let program = parse_file_with_settings(&arena, &file, settings);
    let names = NameResolver::new(&arena).resolve(program);

    let mut reader = ClassReader {
        file: &file,
        names: &names,
        seen: HashSet::new(),
        found: Vec::new(),
        declares: Vec::new(),
    };
    reader.walk_program(program, &mut ());

    // One entry per class: the first line that names it.
    let declared: HashSet<&str> = reader
        .declares
        .iter()
        .map(|declared| declared.class.as_str())
        .collect();
    let mut first: HashMap<&str, u32> = HashMap::new();
    for found in &reader.found {
        let line = first.entry(found.class.as_str()).or_insert(found.line);
        *line = (*line).min(found.line);
    }
    let mut refs: Vec<ClassRef> = first
        .into_iter()
        .filter(|(class, _)| !declared.contains(class))
        .map(|(class, line)| ClassRef {
            class: class.to_owned(),
            line,
        })
        .collect();
    refs.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.class.cmp(&b.class)));
    Analysis {
        declares: reader.declares,
        refs,
    }
}

struct ClassReader<'a, 'arena> {
    file: &'a File,
    names: &'a ResolvedNames<'arena>,
    /// Where each name that was taken starts, so that one name is taken once.
    seen: HashSet<u32>,
    found: Vec<ClassRef>,
    declares: Vec<Declared>,
}

impl<'arena> ClassReader<'_, 'arena> {
    fn full_name(&self, at: &impl HasSpan, raw: &[u8]) -> String {
        let resolved = self.names.resolve(&at.span()).unwrap_or(raw);
        String::from_utf8_lossy(resolved)
            .trim_start_matches('\\')
            .to_owned()
    }

    fn name(&mut self, name: &Identifier<'arena>) {
        if self.seen.insert(name.span().start.offset) {
            let line = self.file.line_number(name.span().start.offset) + 1;
            self.found.push(ClassRef {
                class: self.full_name(name, name.value()),
                line,
            });
        }
    }

    /// The class part of `Foo::bar()`, `new Foo` and the like. Not `self`, `static` or a variable.
    fn class_of(&mut self, expression: &Expression<'arena>) {
        if let Expression::Identifier(name) = expression {
            self.name(name);
        }
    }

    fn declare(
        &mut self,
        kind: DeclaredKind,
        name: &LocalIdentifier<'arena>,
        attributes: &Sequence<'arena, AttributeList<'arena>>,
    ) {
        // A console command can carry its name in an attribute: Laravel's
        // `#[Signature('name {argument}')]` or Symfony's `#[AsCommand(name: 'name')]`.
        let mut command = None;
        let mut names = Vec::new();
        for attribute in attributes.iter().flat_map(|list| list.attributes.iter()) {
            let full = self.full_name(&attribute.name, attribute.name.value());
            let last = full.rsplit('\\').next().unwrap_or_default();
            if matches!(last, "Signature" | "AsCommand")
                && let Some(arguments) = &attribute.argument_list
            {
                let value = arguments
                    .arguments
                    .iter()
                    .find_map(|argument| match argument {
                        PartialArgument::Positional(argument) => string_of(argument.value),
                        PartialArgument::Named(argument) if argument.name.value == b"name" => {
                            string_of(argument.value)
                        }
                        _ => None,
                    });
                command = command.or(value.as_deref().and_then(command_name));
            }
            names.push(full);
        }
        self.declares.push(Declared {
            class: self.full_name(name, name.value),
            kind,
            attributes: names,
            command,
            name_property: None,
        });
    }
}

/// The text of a string literal.
fn string_of(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::Literal(Literal::String(string)) => string.value.map(lossy),
        _ => None,
    }
}

/// The name of a command in its signature: the first word of `name {argument} {--option}`.
fn command_name(signature: &str) -> Option<String> {
    signature.split_whitespace().next().map(str::to_owned)
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for ClassReader<'_, 'arena> {
    fn walk_in_use(&mut self, r#use: &'ast Use<'arena>, _: &mut ()) {
        // `use function` and `use const` do not name classes.
        match &r#use.items {
            UseItems::Sequence(sequence) => {
                sequence.items.iter().for_each(|item| self.name(&item.name))
            }
            UseItems::MixedList(list) => list
                .items
                .iter()
                .filter(|item| item.r#type.is_none())
                .for_each(|item| self.name(&item.item.name)),
            UseItems::TypedSequence(_) | UseItems::TypedList(_) => {}
        }
    }

    fn walk_in_class(&mut self, class: &'ast Class<'arena>, _: &mut ()) {
        self.declare(DeclaredKind::Class, &class.name, &class.attribute_lists);
    }

    fn walk_in_interface(&mut self, interface: &'ast Interface<'arena>, _: &mut ()) {
        self.declare(
            DeclaredKind::Interface,
            &interface.name,
            &interface.attribute_lists,
        );
    }

    fn walk_in_trait(&mut self, r#trait: &'ast Trait<'arena>, _: &mut ()) {
        self.declare(DeclaredKind::Trait, &r#trait.name, &r#trait.attribute_lists);
    }

    fn walk_in_enum(&mut self, r#enum: &'ast Enum<'arena>, _: &mut ()) {
        self.declare(DeclaredKind::Enum, &r#enum.name, &r#enum.attribute_lists);
    }

    // `protected $signature = 'billing:send-invoices {--force}';` in a command class.
    fn walk_in_property_concrete_item(
        &mut self,
        item: &'ast PropertyConcreteItem<'arena>,
        _: &mut (),
    ) {
        let Some(declared) = self.declares.last_mut() else {
            return;
        };
        let value = string_of(item.value).as_deref().and_then(command_name);
        if item.variable.name == b"$signature" && declared.command.is_none() {
            declared.command = value;
        } else if item.variable.name == b"$name" {
            declared.name_property = value;
        }
    }

    fn walk_in_extends(&mut self, extends: &'ast Extends<'arena>, _: &mut ()) {
        extends.types.iter().for_each(|name| self.name(name));
    }

    fn walk_in_implements(&mut self, implements: &'ast Implements<'arena>, _: &mut ()) {
        implements.types.iter().for_each(|name| self.name(name));
    }

    fn walk_in_trait_use(&mut self, trait_use: &'ast TraitUse<'arena>, _: &mut ()) {
        trait_use
            .trait_names
            .iter()
            .for_each(|name| self.name(name));
    }

    fn walk_in_attribute(&mut self, attribute: &'ast Attribute<'arena>, _: &mut ()) {
        self.name(&attribute.name);
    }

    // Parameter, return, property and constant types, also inside ?T, A|B and A&B,
    // and the types of a catch.
    fn walk_in_hint(&mut self, hint: &'ast Hint<'arena>, _: &mut ()) {
        if let Hint::Identifier(name) = hint {
            self.name(name);
        }
    }

    fn walk_in_instantiation(&mut self, instantiation: &'ast Instantiation<'arena>, _: &mut ()) {
        self.class_of(instantiation.class);
    }

    fn walk_in_static_method_call(&mut self, call: &'ast StaticMethodCall<'arena>, _: &mut ()) {
        self.class_of(call.class);
    }

    fn walk_in_static_method_partial_application(
        &mut self,
        call: &'ast StaticMethodPartialApplication<'arena>,
        _: &mut (),
    ) {
        self.class_of(call.class);
    }

    fn walk_in_static_property_access(
        &mut self,
        access: &'ast StaticPropertyAccess<'arena>,
        _: &mut (),
    ) {
        self.class_of(access.class);
    }

    // `Foo::class` and `Foo::CONSTANT`.
    fn walk_in_class_constant_access(
        &mut self,
        access: &'ast ClassConstantAccess<'arena>,
        _: &mut (),
    ) {
        self.class_of(access.class);
    }

    fn walk_in_binary(&mut self, binary: &'ast Binary<'arena>, _: &mut ()) {
        if matches!(binary.operator, BinaryOperator::Instanceof(_)) {
            self.class_of(binary.rhs);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tables_of_a_migration_are_the_literals_its_up_method_creates() {
        let source = r#"<?php
use Illuminate\Support\Facades\Schema as S;
use App\Support\Other;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('orders', function (Blueprint $table) {
            $table->id();
        });
        Schema::connection('reports')->create('order_totals', fn ($table) => null);
        Schema::create($this->table, fn ($table) => null);
        \Illuminate\Support\Facades\Schema::create('refunds', fn ($table) => null);
        S::create('payments', fn ($table) => null);
        S::connection('reports')->create('orders', fn ($table) => null);
        Schema::create(callback: fn ($table) => null, table: 'invoices');
        Other::create('carts', fn ($table) => null);
        Schema::table('carts', fn ($table) => null);
    }

    public function down(): void
    {
        Schema::create('old_orders', fn ($table) => null);
    }
};
"#;
        assert_eq!(
            created_tables(source),
            ["orders", "order_totals", "refunds", "payments", "invoices"]
        );
        // Another class imported as `Schema` is not the facade.
        let other = r#"<?php
use App\Support\TestSchema as Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('orders', fn ($table) => null);
    }
};
"#;
        assert!(created_tables(other).is_empty());
    }

    #[test]
    fn reads_the_name_of_a_console_command() {
        let command = |source: &str| analyze(source.as_bytes()).declares[0].command.clone();
        assert_eq!(
            command(
                "<?php class A extends Command { protected $signature = 'billing:send {--force}'; }"
            ),
            Some("billing:send".to_owned())
        );
        assert_eq!(
            command(
                "<?php use Illuminate\\Console\\Attributes\\Signature;\n#[Signature('billing:retry {id}')] class A {}"
            ),
            Some("billing:retry".to_owned())
        );
        assert_eq!(
            command("<?php #[AsCommand(name: 'billing:prune', description: 'x')] class A {}"),
            Some("billing:prune".to_owned())
        );
        // A signature that is not a literal cannot be read without PHP.
        assert_eq!(
            command("<?php class A { protected $signature = self::NAME; }"),
            None
        );
        assert_eq!(command("<?php class A { public $other = 'x'; }"), None);
        let old = analyze(b"<?php class A extends Command { protected $name = 'billing:old'; }");
        assert_eq!(old.declares[0].command, None);
        assert_eq!(
            old.declares[0].name_property.as_deref(),
            Some("billing:old")
        );
    }

    fn read(source: &str) -> ConfigValue {
        read_config(source.as_bytes()).expect("a return statement")
    }

    #[test]
    fn reads_literals_and_nested_arrays() {
        let config = read(
            r#"<?php
            use Some\Thing;
            return [
                // The last of two equal keys wins, as in PHP.
                'namespace' => 'Old',
                'namespace' => 'Modules',
                "paths" => ['app_folder' => 'app/', 'generate' => true, 'depth' => 2],
                'nothing' => null,
            ];"#,
        );
        assert_eq!(
            config.get("namespace"),
            Some(&ConfigValue::Str("Modules".into()))
        );
        assert_eq!(
            config.at(&["paths", "app_folder"]),
            Some(&ConfigValue::Str("app/".into()))
        );
        assert_eq!(
            config.at(&["paths", "generate"]),
            Some(&ConfigValue::Bool(true))
        );
        assert_eq!(config.at(&["paths", "depth"]), Some(&ConfigValue::Int(2)));
        assert_eq!(config.get("nothing"), Some(&ConfigValue::Null));
        assert_eq!(config.get("missing"), None);
    }

    #[test]
    fn understands_base_path_and_nothing_else_that_needs_php() {
        let config = read(
            r#"<?php return [
                'a' => base_path('Modules'),
                'b' => base_path(),
                'c' => \base_path('x/y'),
                'd' => env('MODULES', 'Modules'),
                'e' => base_path('Modules') . '/more',
                'f' => base_path($dir),
                'g' => __DIR__ . '/../Modules',
            ];"#,
        );
        assert_eq!(
            config.get("a"),
            Some(&ConfigValue::BasePath("Modules".into()))
        );
        assert_eq!(config.get("b"), Some(&ConfigValue::BasePath(String::new())));
        assert_eq!(config.get("c"), Some(&ConfigValue::BasePath("x/y".into())));
        for key in ["d", "e", "f", "g"] {
            assert_eq!(config.get(key), Some(&ConfigValue::Unknown), "key {key}");
        }
    }

    #[test]
    fn a_file_without_a_return_gives_nothing() {
        assert_eq!(read_config(b"<?php $x = 1;"), None);
    }

    #[test]
    fn counts_phpunit_tests_by_name_attribute_and_docblock() {
        let tests = count_tests(
            br#"<?php
            namespace Modules\Billing\Tests\Unit;
            use PHPUnit\Framework\Attributes\Test;
            use PHPUnit\Framework\TestCase;

            class InvoiceTest extends TestCase
            {
                protected function setUp(): void {}

                public function testItAddsLines(): void {}
                public function test_it_rounds(): void {}

                #[Test]
                public function it_numbers_invoices(): void {}

                /** @test */
                public function it_sends_a_copy(): void {}

                /**
                 * Counted once: the name already makes it a test.
                 * @test
                 */
                public function testTwice(): void {}

                /** @testWith [1] is another tag */
                public function helper(): void {}

                public function test(): void {}
                private function testimony(): string { return ''; }
            }"#,
        );
        // testItAddsLines, test_it_rounds, it_numbers_invoices, it_sends_a_copy,
        // testTwice and test. Not setUp, helper, or testimony, which is private.
        assert_eq!(tests, 6);
    }

    #[test]
    fn counts_pest_tests_also_inside_describe() {
        let tests = count_tests(
            br#"<?php
            use Modules\Billing\Models\Invoice;

            it('adds lines', function () { expect(1)->toBe(1); });
            test('rounds', fn () => expect(round(1.5))->toBe(2.0))->group('money');
            describe('numbering', function () {
                it('counts up', function () {});
                it('skips cancelled invoices')->todo();
            });
            beforeEach(function () { $this->invoice = new Invoice; });
            "#,
        );
        assert_eq!(tests, 4);

        assert_eq!(count_tests(b"<?php class Helper { function make() {} }"), 0);
        assert_eq!(count_tests(b""), 0);
        // A call that is not complete is not a test yet.
        assert_eq!(count_tests(b"<?php it("), 0);
    }

    fn names(analysis: &Analysis) -> Vec<(&str, u32)> {
        analysis
            .refs
            .iter()
            .map(|found| (found.class.as_str(), found.line))
            .collect()
    }

    #[test]
    fn names_are_resolved_the_way_php_resolves_them() {
        let analysis = analyze(
            br#"<?php

namespace Modules\Checkout\Services;

use Modules\Billing\Models\Invoice;
use Modules\Billing\Events\{InvoicePaid, InvoiceIssued as Issued};
use Modules\Shipping\Contracts as Shipping;
use function Modules\Billing\helper;

#[\Modules\Users\Attributes\Audited]
final class InvoiceSync extends BaseSync implements Shipping\RateCalculator
{
    use \Modules\Catalog\Concerns\Priced;

    public function sync(?Invoice $invoice, namespace\Options $options): Issued|InvoicePaid
    {
        $paid = new InvoicePaid(1);
        if ($invoice instanceof \Modules\Billing\Models\CreditNote) {
            return \Modules\Inventory\Stock::reserve();
        }
        try {
            return Issued::from(Local::class, self::class, static::make());
        } catch (\Modules\Billing\Exceptions\Late | \RuntimeException $e) {
            throw $e;
        }
    }
}
"#,
        );
        assert_eq!(
            analysis.declares,
            [Declared {
                class: "Modules\\Checkout\\Services\\InvoiceSync".into(),
                kind: DeclaredKind::Class,
                attributes: vec!["Modules\\Users\\Attributes\\Audited".into()],
                command: None,
                name_property: None,
            }]
        );
        assert_eq!(
            names(&analysis),
            [
                ("Modules\\Billing\\Models\\Invoice", 5),
                ("Modules\\Billing\\Events\\InvoiceIssued", 6),
                ("Modules\\Billing\\Events\\InvoicePaid", 6),
                ("Modules\\Shipping\\Contracts", 7),
                ("Modules\\Users\\Attributes\\Audited", 10),
                ("Modules\\Checkout\\Services\\BaseSync", 11),
                ("Modules\\Shipping\\Contracts\\RateCalculator", 11),
                ("Modules\\Catalog\\Concerns\\Priced", 13),
                ("Modules\\Checkout\\Services\\Options", 15),
                ("Modules\\Billing\\Models\\CreditNote", 18),
                ("Modules\\Inventory\\Stock", 19),
                ("Modules\\Checkout\\Services\\Local", 22),
                ("Modules\\Billing\\Exceptions\\Late", 23),
                ("RuntimeException", 23),
            ]
        );
    }

    #[test]
    fn declarations_of_every_kind_are_found() {
        let analysis = analyze(
            br#"<?php
            namespace Modules\Billing\Contracts;
            interface PaymentGateway {}
            trait Charges {}
            enum Currency: string { case Eur = 'eur'; }
            abstract class Gateway implements PaymentGateway { use Charges; }
            "#,
        );
        let declared: Vec<_> = analysis
            .declares
            .iter()
            .map(|d| (d.class.as_str(), d.kind))
            .collect();
        assert_eq!(
            declared,
            [
                (
                    "Modules\\Billing\\Contracts\\PaymentGateway",
                    DeclaredKind::Interface
                ),
                ("Modules\\Billing\\Contracts\\Charges", DeclaredKind::Trait),
                ("Modules\\Billing\\Contracts\\Currency", DeclaredKind::Enum),
                ("Modules\\Billing\\Contracts\\Gateway", DeclaredKind::Class),
            ]
        );
        // What the file declares itself is not a reference.
        assert!(analysis.refs.is_empty(), "{:?}", analysis.refs);
    }

    #[test]
    fn a_file_with_a_syntax_error_gives_what_can_be_read() {
        let analysis = analyze(
            br#"<?php
            namespace Modules\Checkout;
            use Modules\Billing\Models\Invoice;
            class Broken {
                public function a(Invoice $i { return 1 }
            "#,
        );
        assert!(names(&analysis).contains(&("Modules\\Billing\\Models\\Invoice", 3)));
        let _ = analyze(b"");
        let _ = analyze(&[0xff, 0x00, b'<', b'?']);
    }

    #[test]
    fn a_broken_file_does_not_panic() {
        let _ = read_config(b"<?php return ['a' => ");
        let _ = read_config(b"");
        let _ = read_config(&[0xff, 0xfe, 0x00]);
    }
}
