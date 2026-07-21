use std::ops::Range;

use chumsky::prelude::*;

use crate::{Expression, Number, Value};

pub fn value() -> impl Parser<char, Value, Error = Simple<char>> {
    choice((
        eval().map(Value::Expression),
        super::array::array(false).map(Value::UnexpectedArray),
        super::str::string('"').map(Value::Str),
        math().map(Value::Number),
        super::number::number().map(Value::Number),
    ))
}

#[derive(Debug, Clone)]
enum Token {
    Number(String),
    Op(char),
    Identifier(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::Op(op) => write!(f, "{op}"),
            Self::Identifier(id) => write!(f, "{id}"),
        }
    }
}

pub fn math() -> impl Parser<char, Number, Error = Simple<char>> {
    choice((
        just("-").to(Token::Op('-')),
        just("+").to(Token::Op('+')),
        just("*").to(Token::Op('*')),
        just("/").to(Token::Op('/')),
        just("%").to(Token::Op('%')),
        just("^").to(Token::Op('^')),
        just("(").to(Token::Op('(')),
        just(")").to(Token::Op(')')),
        one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")
            .repeated()
            .at_least(1)
            .collect::<String>()
            .map(Token::Identifier),
        super::number::number().map(|n| Token::Number(n.to_string())),
    ))
    .padded()
    .repeated()
    .at_least(2)
    .collect::<Vec<_>>()
    .try_map(|tokens, span: Range<usize>| {
        let expr = tokens
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<String>();
        let number = Number::try_evaluation(&expr, span.clone());
        number.map_or_else(
            || {
                Err(Simple::custom(
                    span,
                    format!("{expr} is not a valid math expression"),
                ))
            },
            Ok,
        )
    })
}

pub fn eval() -> impl Parser<char, Expression, Error = Simple<char>> {
    just("__EVAL".to_string())
        .ignore_then(recursive(|eval| {
            eval.repeated()
                .at_least(1)
                .map(|s| format!("({})", s.join("")))
                .delimited_by(just("(".to_string()), just(")".to_string()))
                .or(none_of("()".to_string())
                    .repeated()
                    .at_least(1)
                    .collect::<String>())
        }))
        .map_with_span(|expr, span| Expression {
            value: expr
                .strip_prefix('(')
                .and_then(|s| s.strip_suffix(')'))
                .expect("eval should be wrapped in brackets")
                .to_string(),
            span,
        })
}

#[cfg(test)]
mod tests {
    use crate::{Number, Str, Value};

    use super::*;

    #[test]
    fn str() {
        assert_eq!(
            value().parse("\"\""),
            Ok(Value::Str(Str {
                value: String::new(),
                span: 0..2
            }))
        );
        assert_eq!(
            value().parse("\"abc\""),
            Ok(Value::Str(Str {
                value: "abc".to_string(),
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("\"abc\"\"def\"\"\""),
            Ok(Value::Str(Str {
                value: "abc\"def\"".to_string(),
                span: 0..12
            }))
        );
        assert_eq!(
            value().parse("\"abc\ndef\""),
            Ok(Value::Str(Str {
                value: "abc\ndef".to_string(),
                span: 0..9
            }))
        );
    }

    #[test]
    fn eval() {
        assert_eq!(
            super::eval().parse("__EVAL(1 + 2)"),
            Ok(Expression {
                value: "1 + 2".to_string(),
                span: 0..13
            })
        );
        assert_eq!(
            super::eval().parse("__EVAL(2 * (1 + 1))"),
            Ok(Expression {
                value: "2 * (1 + 1)".to_string(),
                span: 0..19
            })
        );
    }

    #[test]
    fn number() {
        assert_eq!(
            value().parse("123"),
            Ok(Value::Number(Number::Int32 {
                value: 123,
                span: 0..3
            }))
        );
        assert_eq!(
            value().parse("123.456"),
            Ok(Value::Number(Number::Float32 {
                value: 123.456,
                span: 0..7
            }))
        );
        assert_eq!(
            value().parse("123.456e-7"),
            Ok(Value::Number(Number::Float32 {
                value: 0.000_012_345_6,
                span: 0..10
            }))
        );
        assert_eq!(
            value().parse("123.456e+7"),
            Ok(Value::Number(Number::Float32 {
                value: 1_234_560_000.0,
                span: 0..10
            }))
        );
        assert_eq!(
            value().parse("123.456e7"),
            Ok(Value::Number(Number::Float32 {
                value: 1_234_560_000.0,
                span: 0..9
            }))
        );
        assert_eq!(
            value().parse("123.456e+"),
            Ok(Value::Number(Number::Float32 {
                value: 123.456,
                span: 0..7
            }))
        );
        assert_eq!(
            value().parse("123.456e-"),
            Ok(Value::Number(Number::Float32 {
                value: 123.456,
                span: 0..7
            }))
        );
        assert_eq!(
            value().parse("123.456e"),
            Ok(Value::Number(Number::Float32 {
                value: 123.456,
                span: 0..7
            }))
        );
        assert_eq!(
            value().parse("123.456e+abc"),
            Ok(Value::Number(Number::Float32 {
                value: 123.456,
                span: 0..7
            }))
        );
        assert_eq!(
            value().parse("123.456e-abc"),
            Ok(Value::Number(Number::Float32 {
                value: 123.456,
                span: 0..7
            }))
        );
        assert_eq!(
            value().parse("123.456eabc"),
            Ok(Value::Number(Number::Float32 {
                value: 123.456,
                span: 0..7
            }))
        );
    }

    #[test]
    fn math() {
        assert_eq!(
            value().parse("1 + 2"),
            Ok(Value::Number(Number::Int32 {
                value: 3,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("1 - 2"),
            Ok(Value::Number(Number::Int32 {
                value: -1,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("1 * 2"),
            Ok(Value::Number(Number::Int32 {
                value: 2,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("1 / 2"),
            Ok(Value::Number(Number::Float32 {
                value: 0.5,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("1 % 2"),
            Ok(Value::Number(Number::Int32 {
                value: 1,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("1 ^ 2"),
            Ok(Value::Number(Number::Int32 {
                value: 1,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("1 + 2 * 3"),
            Ok(Value::Number(Number::Int32 {
                value: 7,
                span: 0..9
            }))
        );
        assert_eq!(
            value().parse("(1 + 2) * 3"),
            Ok(Value::Number(Number::Int32 {
                value: 9,
                span: 0..11
            }))
        );
        assert_eq!(
            value().parse("1 + 2 * 3 + 4"),
            Ok(Value::Number(Number::Int32 {
                value: 11,
                span: 0..13
            }))
        );
        assert_eq!(
            value().parse("1 + 2 * (3 + 4)"),
            Ok(Value::Number(Number::Int32 {
                value: 15,
                span: 0..15
            }))
        );
        assert_eq!(
            value().parse("2 ^ 3"),
            Ok(Value::Number(Number::Int32 {
                value: 8,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("10 % 3"),
            Ok(Value::Number(Number::Int32 {
                value: 1,
                span: 0..6
            }))
        );
        assert_eq!(
            value().parse("10 % 3 + 2"),
            Ok(Value::Number(Number::Int32 {
                value: 3,
                span: 0..10
            }))
        );
        assert_eq!(
            value().parse("-0.01*0.5"),
            Ok(Value::Number(Number::Float32 {
                value: -0.005,
                span: 0..9
            }))
        );
        assert_eq!(
            value().parse("-(0.01*0.5)"),
            Ok(Value::Number(Number::Float32 {
                value: -0.005,
                span: 0..11
            }))
        );
        assert_eq!(
            value().parse("(-0.01)"),
            Ok(Value::Number(Number::Float32 {
                value: -0.01,
                span: 0..7
            }))
        );
        assert_eq!(
            value().parse("1-2-3"),
            Ok(Value::Number(Number::Int32 {
                value: -4,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("1 - 2 - 3"),
            Ok(Value::Number(Number::Int32 {
                value: -4,
                span: 0..9
            }))
        );
    }

    #[test]
    fn math_functions() {
        assert_eq!(
            super::math().parse("rad(180)"),
            Ok(Number::Float32 {
                value: std::f64::consts::PI as f32,
                span: 0..8
            })
        );
        assert_eq!(
            super::math().parse("rad 180"),
            Ok(Number::Float32 {
                value: std::f64::consts::PI as f32,
                span: 0..7
            })
        );
    }

    #[test]
    fn not_math() {
        // Incomplete math expression - trailing operator
        assert!(
            value()
                .padded()
                .then_ignore(end())
                .parse("1 + 2 +")
                .is_err(),
        );

        // Leading operator without operand
        assert!(
            value()
                .padded()
                .then_ignore(end())
                .parse("+ 1 + 2")
                .is_err(),
        );

        // Double operators
        assert!(
            value()
                .padded()
                .then_ignore(end())
                .parse("1 + + 2")
                .is_err(),
        );

        // Unmatched parentheses
        assert!(value().padded().then_ignore(end()).parse("(1 + 2").is_err(),);
        assert!(value().padded().then_ignore(end()).parse("1 + 2)").is_err(),);

        // Invalid function
        assert!(
            value()
                .padded()
                .then_ignore(end())
                .parse("foo(123)")
                .is_err(),
        );

        // Just an identifier
        assert!(value().padded().then_ignore(end()).parse("xyz").is_err(),);
    }
}
