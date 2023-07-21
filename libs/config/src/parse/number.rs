use chumsky::prelude::*;

use crate::Number;

pub fn number() -> impl Parser<char, Number, Error = Simple<char>> {
    just('-')
        .or_not()
        .then(choice((
            number_float_exponent().map_with_span(|value, span| Number::Float32 { value, span }),
            number_float_basic().map_with_span(|value, span| Number::Float32 { value, span }),
            number_hex().map_with_span(|value, span| {
                if value > i64::from(i32::MAX) {
                    Number::Int64 { value, span }
                } else {
                    Number::Int32 {
                        value: value as i32,
                        span,
                    }
                }
            }),
            number_int().map_with_span(|value, span| {
                if value > i64::from(i32::MAX) {
                    Number::Int64 { value, span }
                } else {
                    Number::Int32 {
                        value: value as i32,
                        span,
                    }
                }
            }),
        )))
        .map(|(sign, value)| match sign {
            Some(_) => value.negate(),
            None => value,
        })
}

fn number_hex() -> impl Parser<char, i64, Error = Simple<char>> {
    let digits = one_of("0123456789abcdefABCDEF").repeated().at_least(1);
    just("0x")
        .ignore_then(digits)
        .collect::<String>()
        .map(|value| i64::from_str_radix(&value, 16))
        .try_map(error_map)
}

fn number_int() -> impl Parser<char, i64, Error = Simple<char>> {
    number_digits()
        .collect::<String>()
        //.then_ignore(just('.').not().rewind())
        .from_str::<i64>()
        .try_map(error_map)
}

fn number_float_exponent() -> impl Parser<char, f32, Error = Simple<char>> {
    number_digits()
        .chain(just('.'))
        .or_not()
        .chain::<char, _, _>(number_digits())
        .chain::<char, _, _>(one_of("eE"))
        .chain::<char, _, _>(one_of("-+").or_not())
        .chain::<char, _, _>(number_digits())
        .collect::<String>()
        .from_str::<f32>()
        .try_map(error_map)
}

fn number_float_basic() -> impl Parser<char, f32, Error = Simple<char>> {
    number_digits()
        .or_not()
        .chain::<char, _, _>(just('.'))
        .chain::<char, _, _>(number_digits())
        .collect::<String>()
        .from_str::<f32>()
        .try_map(error_map)
}

fn number_digits() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    one_of("0123456789").repeated().at_least(1)
}

#[inline]
#[allow(clippy::result_large_err)] // todo
fn error_map<T, E, I>(result: Result<T, E>, span: std::ops::Range<usize>) -> Result<T, Simple<I>>
where
    E: ToString,
    I: std::hash::Hash + Eq,
{
    result.map_err(|err| Simple::custom(span, err))
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;

    #[test]
    fn number_digits() {
        assert_eq!(super::number_digits().parse("123"), Ok(vec!['1', '2', '3']));
        assert_eq!(
            super::number_digits().parse("123abc"),
            Ok(vec!['1', '2', '3'])
        );
        assert!(super::number_digits().parse("abc").is_err());
    }

    #[test]
    fn number_int() {
        assert_eq!(super::number_int().parse("123"), Ok(123));
        assert_eq!(super::number_int().parse("123abc"), Ok(123));
        assert_eq!(super::number_int().parse("123.456"), Ok(123));
        assert_eq!(super::number_int().parse("123.456abc"), Ok(123));
        assert!(super::number_int().parse("abc").is_err());
    }

    #[test]
    fn number_hex() {
        assert_eq!(super::number_hex().parse("0x123"), Ok(0x123));
        assert_eq!(super::number_hex().parse("0x123abc"), Ok(0x0012_3ABC));
        assert_eq!(super::number_hex().parse("0x123.456"), Ok(0x123));
        assert!(super::number_hex().parse("abc").is_err());
    }

    #[test]
    fn number_float_basic() {
        assert_eq!(super::number_float_basic().parse("123.0"), Ok(123.0));
        assert_eq!(super::number_float_basic().parse("123.0abc"), Ok(123.0));
        assert_eq!(super::number_float_basic().parse("123.456"), Ok(123.456));
        assert_eq!(super::number_float_basic().parse("123.456abc"), Ok(123.456));
        assert!(super::number_float_basic().parse("abc").is_err());
    }

    #[test]
    fn number_float_exponent() {
        assert_eq!(
            super::number_float_exponent().parse("123.456e-2"),
            Ok(1.23456)
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456e+2"),
            Ok(12345.6)
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456e2"),
            Ok(12345.6)
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456E-2"),
            Ok(1.23456)
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456E+2"),
            Ok(12345.6)
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456E2"),
            Ok(12345.6)
        );
        assert!(super::number_float_exponent().parse("abc").is_err());
    }
}
