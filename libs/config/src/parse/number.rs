use chumsky::prelude::*;

use crate::{Number, parse::ParseError};

pub fn number<'src>() -> impl Parser<'src, &'src str, Number, ParseError<'src>> + Clone {
    just('-')
        .or_not()
        .then(choice((
            number_float_exponent().map(Number::Float32),
            number_float_basic().map(Number::Float32),
            number_hex().map(|value| {
                if value > i64::from(i32::MAX) {
                    Number::Int64(value)
                } else {
                    Number::Int32(value as i32)
                }
            }),
            number_int().map(|value| {
                if value > i64::from(i32::MAX) {
                    Number::Int64(value)
                } else {
                    Number::Int32(value as i32)
                }
            }),
        )))
        .map(|(sign, value)| match sign {
            Some(_) => value.negate(),
            None => value,
        })
}

fn number_hex<'src>() -> impl Parser<'src, &'src str, i64, ParseError<'src>> + Clone {
    let digits = one_of("0123456789abcdefABCDEF")
        .repeated()
        .at_least(1)
        .to_slice();
    just("0x")
        .ignore_then(digits)
        .map(|value| i64::from_str_radix(value, 16))
        .try_map(error_map)
}

fn number_int<'src>() -> impl Parser<'src, &'src str, i64, ParseError<'src>> + Clone {
    number_digits().from_str::<i64>().try_map(error_map)
}

fn number_float_exponent<'src>() -> impl Parser<'src, &'src str, f32, ParseError<'src>> + Clone {
    number_digits()
        .then(just('.'))
        .or_not()
        .then(number_digits())
        .then(one_of("eE"))
        .then(one_of("-+").or_not())
        .then(number_digits())
        .to_slice()
        .from_str::<f32>()
        .try_map(error_map)
}

fn number_float_basic<'src>() -> impl Parser<'src, &'src str, f32, ParseError<'src>> + Clone {
    number_digits()
        .or_not()
        .then(just('.'))
        .then(number_digits())
        .to_slice()
        .from_str::<f32>()
        .try_map(error_map)
}

fn number_digits<'src>() -> impl Parser<'src, &'src str, &'src str, ParseError<'src>> + Clone {
    one_of("0123456789").repeated().at_least(1).to_slice()
}

#[inline]
fn error_map<'src, T, E, I>(result: Result<T, E>, span: SimpleSpan) -> Result<T, Rich<'src, I>>
where
    E: ToString,
    I: std::hash::Hash + Eq,
{
    result.map_err(|err| Rich::custom(span, err))
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;

    use crate::Number;

    #[test]
    fn number_digits() {
        assert_eq!(super::number_digits().parse("123").unwrap(), "123");
        assert!(super::number_digits().parse("123abc").has_errors());
        assert!(super::number_digits().parse("abc").has_errors());
    }

    #[test]
    fn number_int() {
        assert_eq!(super::number_int().parse("123").unwrap(), 123);
        assert!(super::number_int().parse("123abc").has_errors());
        assert!(super::number_int().parse("123.456").has_errors());
        assert!(super::number_int().parse("123.456abc").has_errors());
        assert!(super::number_int().parse("abc").has_errors());
    }

    #[test]
    fn number_hex() {
        assert_eq!(super::number_hex().parse("0x123").unwrap(), 0x123);
        assert_eq!(super::number_hex().parse("0x123abc").unwrap(), 0x0012_3ABC);
        assert!(super::number_hex().parse("0x123.456").has_errors());
        assert!(super::number_hex().parse("abc").has_errors());
    }

    #[allow(clippy::float_cmp)]
    #[test]
    fn number_float_basic() {
        assert_eq!(super::number_float_basic().parse("123.0").unwrap(), 123.0);
        assert!(super::number_float_basic().parse("123.0abc").has_errors());
        assert_eq!(
            super::number_float_basic().parse("123.456").unwrap(),
            123.456
        );
        assert!(super::number_float_basic().parse("123.456abc").has_errors());
        assert_eq!(super::number_float_basic().parse("0.01").unwrap(), 0.01);
        assert_eq!(super::number_float_basic().parse("0.5").unwrap(), 0.5);
        assert!(super::number_float_basic().parse("abc").has_errors());
    }

    #[test]
    fn number_float_negative() {
        assert_eq!(
            super::number().parse("-123.0").unwrap(),
            Number::Float32(-123.0)
        );
        assert!(super::number().parse("-123.0abc").has_errors());
        assert_eq!(
            super::number().parse("-123.456").unwrap(),
            Number::Float32(-123.456)
        );
        assert!(super::number().parse("-123.456abc").has_errors());
        assert_eq!(
            super::number().parse("-0.01").unwrap(),
            Number::Float32(-0.01)
        );
        assert_eq!(
            super::number().parse("-0.5").unwrap(),
            Number::Float32(-0.5)
        );
        assert!(super::number().parse("-abc").has_errors());
    }

    #[allow(clippy::float_cmp)]
    #[test]
    fn number_float_exponent() {
        assert_eq!(
            super::number_float_exponent().parse("123.456e-2").unwrap(),
            1.23456
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456e+2").unwrap(),
            12345.6
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456e2").unwrap(),
            12345.6
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456E-2").unwrap(),
            1.23456
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456E+2").unwrap(),
            12345.6
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456E2").unwrap(),
            12345.6
        );
        assert!(super::number_float_exponent().parse("abc").has_errors());
    }
}
