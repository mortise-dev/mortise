//! The only module that touches the PHP parser.
//!
//! The parser's API changes between minor versions, so nothing outside this
//! file names one of its types. What leaves here is owned, plain data.

use mago_allocator::LocalArena;
use mago_database::file::FileId;
use mago_syntax::cst::{
    Argument, ArrayElement, Call, Expression, Literal, Statement, TokenSeparatedSequence,
};
use mago_syntax::parser::parse_file_content;

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
    fn a_broken_file_does_not_panic() {
        let _ = read_config(b"<?php return ['a' => ");
        let _ = read_config(b"");
        let _ = read_config(&[0xff, 0xfe, 0x00]);
    }
}
