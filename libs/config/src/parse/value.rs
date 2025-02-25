use chumsky::prelude::*;

use crate::{Expression, Number, Value};

pub fn value<'a>() -> impl Parser<'a, &'a str, Value, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        eval().map(Value::Expression),
        super::array::array(false).map(Value::UnexpectedArray),
        super::str::string().map(Value::Str),
        math().map(Value::Number),
        super::number::number().map(Value::Number),
    ))
}

pub fn math<'a>() -> impl Parser<'a, &'a str, Number, extra::Err<Rich<'a, char>>> + Clone {
    enum MathObject {
        Valid(String),
        Invalid(String, SimpleSpan),
    }
    impl MathObject {
        fn into_string(self) -> String {
            match self {
                Self::Invalid(s, _) | Self::Valid(s) => s,
            }
        }
    }
    choice((
        choice((
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
        .map(MathObject::Valid),
        super::number::number().map(|n| MathObject::Valid(n.to_string())),
        none_of(" ;,}".to_string())
            .repeated()
            .at_least(1)
            .collect::<String>()
            .map_with(|s, extra| MathObject::Invalid(s, extra.span())),
    ))
    .repeated()
    .at_least(2)
    .collect::<Vec<_>>()
    .try_map(|exprs, span| {
        if let Some(err) = exprs
            .iter()
            .map(|expr| match expr {
                MathObject::Invalid(_, span) => Some(Number::InvalidMath {
                    span: span.into_range(),
                }),
                MathObject::Valid(_) => None,
            })
            .find(std::option::Option::is_some)
        {
            return Ok(err.expect("was found to be some"));
        }
        let expr_string = exprs
            .into_iter()
            .map(MathObject::into_string)
            .collect::<Vec<_>>()
            .join(" ");
        let number = Number::try_evaulation(&expr_string, span.into_range());
        number.map_or_else(
            || {
                Ok(Number::InvalidMath {
                    span: span.into_range(),
                })
            },
            Ok,
        )
    })
}

pub fn eval<'a>() -> impl Parser<'a, &'a str, Expression, extra::Err<Rich<'a, char>>> + Clone {
    just("__EVAL".to_string())
        .ignore_then(recursive(|eval| {
            eval.repeated()
                .at_least(1)
                .collect::<Vec<_>>()
                .map(|s| format!("({})", s.join("")))
                .delimited_by(just("(".to_string()), just(")".to_string()))
                .or(none_of("()".to_string())
                    .repeated()
                    .at_least(1)
                    .collect::<String>())
        }))
        .map_with(|expr, extra| Expression {
            value: expr
                .strip_prefix('(')
                .and_then(|s| s.strip_suffix(')'))
                .expect("eval should be wrapped in brackets")
                .to_string(),
            span: (extra.span() as SimpleSpan).into_range(),
        })
}

#[cfg(test)]
mod tests {
    use crate::{Number, Str, Value};

    use super::*;

    #[test]
    fn str() {
        assert_eq!(
            value().parse("\"\"").output(),
            Some(&Value::Str(Str {
                value: String::new(),
                span: 0..2
            }))
        );
        assert_eq!(
            value().parse("\"abc\"").output(),
            Some(&Value::Str(Str {
                value: "abc".to_string(),
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("\"abc\"\"def\"\"\"").output(),
            Some(&Value::Str(Str {
                value: "abc\"def\"".to_string(),
                span: 0..12
            }))
        );
        assert_eq!(
            value().parse("\"abc\ndef\"").output(),
            Some(&Value::Str(Str {
                value: "abc\ndef".to_string(),
                span: 0..9
            }))
        );
    }

    #[test]
    fn eval() {
        assert_eq!(
            super::eval().parse("__EVAL(1 + 2)").output(),
            Some(&Expression {
                value: "1 + 2".to_string(),
                span: 0..13
            })
        );
        assert_eq!(
            super::eval().parse("__EVAL(2 * (1 + 1))").output(),
            Some(&Expression {
                value: "2 * (1 + 1)".to_string(),
                span: 0..19
            })
        );
    }

    #[test]
    fn number() {
        assert_eq!(
            value().parse("123").output(),
            Some(&Value::Number(Number::Int32 {
                value: 123,
                span: 0..3
            }))
        );
        assert_eq!(
            value().parse("123.456").output(),
            Some(&Value::Number(Number::Float32 {
                value: 123.456,
                span: 0..7
            }))
        );
        assert_eq!(
            value().parse("123.456e-7").output(),
            Some(&Value::Number(Number::Float32 {
                value: 0.000_012_345_6,
                span: 0..10
            }))
        );
        assert_eq!(
            value().parse("123.456e+7").output(),
            Some(&Value::Number(Number::Float32 {
                value: 1_234_560_000.0,
                span: 0..10
            }))
        );
        assert_eq!(
            value().parse("123.456e7").output(),
            Some(&Value::Number(Number::Float32 {
                value: 1_234_560_000.0,
                span: 0..9
            }))
        );
        assert_eq!(
            value().parse("123.456e+").output(),
            Some(&Value::Number(Number::Float32 {
                value: 123.456,
                span: 0..7
            }))
        );
        assert_eq!(
            value().parse("123.456e-").output(),
            Some(&Value::Number(Number::Float32 {
                value: 123.456,
                span: 0..7
            }))
        );
        assert_eq!(
            value().parse("123.456e").output(),
            Some(&Value::Number(Number::Float32 {
                value: 123.456,
                span: 0..7
            }))
        );
    }

    #[test]
    fn math() {
        assert_eq!(
            value().parse("1 + 2").output(),
            Some(&Value::Number(Number::Int32 {
                value: 3,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("1 - 2").output(),
            Some(&Value::Number(Number::Int32 {
                value: -1,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("1 * 2").output(),
            Some(&Value::Number(Number::Int32 {
                value: 2,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("1 / 2").output(),
            Some(&Value::Number(Number::Float32 {
                value: 0.5,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("1 % 2").output(),
            Some(&Value::Number(Number::Int32 {
                value: 1,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("1 ^ 2").output(),
            Some(&Value::Number(Number::Int32 {
                value: 1,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("1 + 2 * 3").output(),
            Some(&Value::Number(Number::Int32 {
                value: 7,
                span: 0..9
            }))
        );
        assert_eq!(
            value().parse("(1 + 2) * 3").output(),
            Some(&Value::Number(Number::Int32 {
                value: 9,
                span: 0..11
            }))
        );
        assert_eq!(
            value().parse("1 + 2 * 3 + 4").output(),
            Some(&Value::Number(Number::Int32 {
                value: 11,
                span: 0..13
            }))
        );
        assert_eq!(
            value().parse("1 + 2 * (3 + 4)").output(),
            Some(&Value::Number(Number::Int32 {
                value: 15,
                span: 0..15
            }))
        );
        assert_eq!(
            value().parse("2 ^ 3").output(),
            Some(&Value::Number(Number::Int32 {
                value: 8,
                span: 0..5
            }))
        );
        assert_eq!(
            value().parse("10 % 3").output(),
            Some(&Value::Number(Number::Int32 {
                value: 1,
                span: 0..6
            }))
        );
    }

    #[test]
    fn invalid_math() {
        assert_eq!(
            value().parse("1 + 2 +").output(),
            Some(&Value::Number(Number::InvalidMath { span: 0..7 }))
        );
        assert_eq!(
            value().parse("1 + two").output(),
            Some(&Value::Number(Number::InvalidMath { span: 4..7 }))
        );
    }
}
