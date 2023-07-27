use std::ops::Range;

use chumsky::prelude::*;

use crate::{Number, Value};

pub fn value() -> impl Parser<char, Value, Error = Simple<char>> {
    choice((
        super::array::array(false).map(Value::UnexpectedArray),
        super::str::string('"').map(Value::Str),
        math().map(Value::Number),
        super::number::number().map(Value::Number),
    ))
}

pub fn math() -> impl Parser<char, Number, Error = Simple<char>> {
    choice((
        super::number::number().map(|n| n.to_string()),
        just("-".to_string()),
        just("+".to_string()),
        just("*".to_string()),
        just("/".to_string()),
        just("%".to_string()),
        just("^".to_string()),
        just("(".to_string()),
        just(")".to_string()),
        just(" ".to_string()),
    ))
    .repeated()
    .at_least(2)
    .collect::<String>()
    .map(|s| s.trim().to_string())
    .validate(|expr, span: Range<usize>, emit| {
        let number = Number::try_evaulation(&expr, span.clone());
        if number.is_none() {
            println!("{expr} is not a valid math expression");
            emit(Simple::custom(
                span,
                format!("{expr} is not a valid math expression"),
            ));
        }
        number
    })
    .map(|number| number.expect("math expression should be valid"))
}

#[cfg(test)]
mod tests {
    use crate::{Number, Str, Value};

    use super::*;

    #[test]
    fn test_str() {
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
    fn test_number() {
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
    fn test_math() {
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
    }
}
