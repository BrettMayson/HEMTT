use std::sync::Arc;

use chumsky::prelude::*;

use crate::{
    Expression, Number, Value,
    parse::{ParseError, raise_span},
};

pub fn value<'src>() -> impl Parser<'src, &'src str, Spanned<Value>, ParseError<'src>> + Clone {
    choice((
        eval().map(Value::Expression).spanned(),
        super::array::array(false).map(|i| raise_span(i, Value::UnexpectedArray)),
        super::str::string('"').map(Value::Str).spanned(),
        math().map(Value::Number).spanned(),
        super::number::number().map(Value::Number).spanned(),
    ))
}

#[derive(Debug, Clone)]
enum Token {
    Number(String),
    Op(char),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::Op(op) => write!(f, "{op}"),
        }
    }
}

pub fn math<'src>() -> impl Parser<'src, &'src str, Number, ParseError<'src>> + Clone {
    choice((
        just("-").to(Token::Op('-')),
        just("+").to(Token::Op('+')),
        just("*").to(Token::Op('*')),
        just("/").to(Token::Op('/')),
        just("%").to(Token::Op('%')),
        just("^").to(Token::Op('^')),
        just("(").to(Token::Op('(')),
        just(")").to(Token::Op(')')),
        super::number::number().map(|n| Token::Number(n.to_string())),
    ))
    .padded()
    .repeated()
    .at_least(2)
    .collect::<Vec<_>>()
    .try_map(|tokens: Vec<Token>, span: SimpleSpan| {
        let has_operator = tokens.iter().any(|t| matches!(t, Token::Op(_)));
        if !has_operator {
            return Err(Rich::custom(
                span,
                "math expression must contain at least one operator",
            ));
        }
        let expr = tokens
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<String>();
        let number = Number::try_evaluation(&expr);
        number.map_or_else(
            || {
                Err(Rich::custom(
                    span,
                    format!("{expr} is not a valid math expression"),
                ))
            },
            Ok,
        )
    })
}

pub fn eval<'src>() -> impl Parser<'src, &'src str, Expression, ParseError<'src>> + Clone {
    just("__EVAL")
        .ignore_then(recursive(|eval| {
            eval.repeated()
                .at_least(1)
                .to_slice()
                .map(|s: &str| format!("({s})"))
                .delimited_by(just("("), just(")"))
                .or(none_of("()").repeated().at_least(1).collect::<String>())
        }))
        .map(|expr| {
            Expression(Arc::from(
                expr.strip_prefix('(')
                    .and_then(|s| s.strip_suffix(')'))
                    .expect("eval should be wrapped in brackets"),
            ))
        })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{Number, Str, Value};

    use super::*;

    #[test]
    fn str() {
        assert_eq!(
            value().parse("\"\"").unwrap().inner,
            Value::Str(Str(Arc::from(""))),
        );
        assert_eq!(
            value().parse("\"abc\"").unwrap().inner,
            Value::Str(Str(Arc::from("abc"))),
        );
        assert_eq!(
            value().parse("\"abc\"\"def\"\"\"").unwrap().inner,
            Value::Str(Str(Arc::from("abc\"def\""))),
        );
        assert_eq!(
            value().parse("\"abc\ndef\"").unwrap().inner,
            Value::Str(Str(Arc::from("abc\ndef"))),
        );
    }

    #[test]
    fn eval() {
        assert_eq!(
            super::eval().parse("__EVAL(1 + 2)").unwrap(),
            Expression(Arc::from("1 + 2")),
        );
        assert_eq!(
            super::eval().parse("__EVAL(2 * (1 + 1))").unwrap(),
            Expression(Arc::from("2 * (1 + 1)")),
        );
    }

    #[test]
    fn number() {
        assert_eq!(
            value().parse("123").unwrap().inner,
            Value::Number(Number::Int32(123)),
        );
        assert_eq!(
            value().parse("123.456").unwrap().inner,
            Value::Number(Number::Float32(123.456)),
        );
        assert_eq!(
            value().parse("123.456e-7").unwrap().inner,
            Value::Number(Number::Float32(0.000_012_345_6)),
        );
        assert_eq!(
            value().parse("123.456e+7").unwrap().inner,
            Value::Number(Number::Float32(1_234_560_000.0)),
        );
        assert_eq!(
            value().parse("123.456e7").unwrap().inner,
            Value::Number(Number::Float32(1_234_560_000.0)),
        );
        assert!(value().parse("123.456e+").has_errors());
        assert!(value().parse("123.456e-").has_errors());
        assert!(value().parse("123.456e").has_errors());
        assert!(value().parse("123.456e+abc").has_errors());
        assert!(value().parse("123.456e-abc").has_errors());
        assert!(value().parse("123.456eabc").has_errors());
    }

    #[test]
    fn math() {
        assert_eq!(
            value().parse("1 + 2").unwrap().inner,
            Value::Number(Number::Int32(3)),
        );
        assert_eq!(
            value().parse("1 - 2").unwrap().inner,
            Value::Number(Number::Int32(-1)),
        );
        assert_eq!(
            value().parse("1 * 2").unwrap().inner,
            Value::Number(Number::Int32(2)),
        );
        assert_eq!(
            value().parse("1 / 2").unwrap().inner,
            Value::Number(Number::Float32(0.5)),
        );
        assert_eq!(
            value().parse("1 % 2").unwrap().inner,
            Value::Number(Number::Int32(1)),
        );
        assert_eq!(
            value().parse("1 ^ 2").unwrap().inner,
            Value::Number(Number::Int32(1)),
        );
        assert_eq!(
            value().parse("1 + 2 * 3").unwrap().inner,
            Value::Number(Number::Int32(7)),
        );
        assert_eq!(
            value().parse("(1 + 2) * 3").unwrap().inner,
            Value::Number(Number::Int32(9)),
        );
        assert_eq!(
            value().parse("1 + 2 * 3 + 4").unwrap().inner,
            Value::Number(Number::Int32(11)),
        );
        assert_eq!(
            value().parse("1 + 2 * (3 + 4)").unwrap().inner,
            Value::Number(Number::Int32(15)),
        );
        assert_eq!(
            value().parse("2 ^ 3").unwrap().inner,
            Value::Number(Number::Int32(8)),
        );
        assert_eq!(
            value().parse("10 % 3").unwrap().inner,
            Value::Number(Number::Int32(1)),
        );
        assert_eq!(
            value().parse("10 % 3 + 2").unwrap().inner,
            Value::Number(Number::Int32(3)),
        );
        assert_eq!(
            value().parse("-0.01*0.5").unwrap().inner,
            Value::Number(Number::Float32(-0.005)),
        );
        assert_eq!(
            value().parse("-(0.01*0.5)").unwrap().inner,
            Value::Number(Number::Float32(-0.005)),
        );
        assert_eq!(
            value().parse("(-0.01)").unwrap().inner,
            Value::Number(Number::Float32(-0.01)),
        );
        assert_eq!(
            value().parse("1-2-3").unwrap().inner,
            Value::Number(Number::Int32(-4)),
        );
        assert_eq!(
            value().parse("1 - 2 - 3").unwrap().inner,
            Value::Number(Number::Int32(-4)),
        );
    }
}
