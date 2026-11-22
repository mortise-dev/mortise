//! The only module that touches the PHP parser.
//!
//! The parser's API changes between minor versions, so nothing outside this
//! file names one of its types. What leaves here is owned, plain data.

use mago_allocator::LocalArena;
use mago_database::file::FileId;
use mago_syntax::cst::{
    Argument, ArrayElement, Call, Expression, FunctionCall, Literal, Method, Modifier, Statement,
    TokenSeparatedSequence, TriviaKind,
};
use mago_syntax::parser::parse_file_content;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn read(source: &str) -> ConfigValue {
        read_config(source.as_bytes()).expect("a return statement")
    }

    #[test]
    fn reads_literals_and_nested_arrays() {
        let config = read(
            r#"<?php
            use Some\Thing;
            return [
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
    fn the_last_of_two_equal_keys_wins() {
        let config = read("<?php return ['a' => 'one', 'a' => 'two'];");
        assert_eq!(config.get("a"), Some(&ConfigValue::Str("two".into())));
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
    }

    #[test]
    fn a_file_without_tests_counts_none() {
        assert_eq!(count_tests(b"<?php class Helper { function make() {} }"), 0);
        assert_eq!(count_tests(b""), 0);
        // A call that is not complete is not a test yet.
        assert_eq!(count_tests(b"<?php it("), 0);
    }

    #[test]
    fn a_broken_file_does_not_panic() {
        let _ = read_config(b"<?php return ['a' => ");
        let _ = read_config(b"");
        let _ = read_config(&[0xff, 0xfe, 0x00]);
    }
}
