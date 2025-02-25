use chumsky::prelude::*;

use crate::Number;

pub fn number<'a>() -> impl Parser<'a, &'a str, Number, extra::Err<Rich<'a, char>>> + Clone {
    just('-')
        .or_not()
        .then(choice((
            number_float_exponent().map_with(|value, extra| Number::Float32 {
                value,
                span: (extra.span() as SimpleSpan).into_range(),
            }),
            number_float_basic().map_with(|value, extra| Number::Float32 {
                value,
                span: (extra.span() as SimpleSpan).into_range(),
            }),
            number_hex().map_with(|value, extra| {
                if value > i64::from(i32::MAX) {
                    Number::Int64 {
                        value,
                        span: (extra.span() as SimpleSpan).into_range(),
                    }
                } else {
                    Number::Int32 {
                        value: value as i32,
                        span: (extra.span() as SimpleSpan).into_range(),
                    }
                }
            }),
            number_int().map_with(|value, extra| {
                if value > i64::from(i32::MAX) {
                    Number::Int64 {
                        value,
                        span: (extra.span() as SimpleSpan).into_range(),
                    }
                } else {
                    Number::Int32 {
                        value: value as i32,
                        span: (extra.span() as SimpleSpan).into_range(),
                    }
                }
            }),
        )))
        .map(|(sign, value)| match sign {
            Some(_) => value.negate(),
            None => value,
        })
}

fn number_hex<'a>() -> impl Parser<'a, &'a str, i64, extra::Err<Rich<'a, char>>> + Clone {
    let digits = one_of("0123456789abcdefABCDEF")
        .repeated()
        .at_least(1)
        .to_slice();
    just("0x")
        .ignore_then(digits)
        .map(|value| i64::from_str_radix(value, 16))
        .try_map(error_map)
}

fn number_int<'a>() -> impl Parser<'a, &'a str, i64, extra::Err<Rich<'a, char>>> + Clone {
    number_digits()
        .to_slice()
        .from_str::<i64>()
        .try_map(error_map)
}

fn number_float_exponent<'a>() -> impl Parser<'a, &'a str, f32, extra::Err<Rich<'a, char>>> + Clone
{
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

fn number_float_basic<'a>() -> impl Parser<'a, &'a str, f32, extra::Err<Rich<'a, char>>> + Clone {
    number_digits()
        .or_not()
        .then(just('.'))
        .then(number_digits())
        .to_slice()
        .from_str::<f32>()
        .try_map(error_map)
}

fn number_digits<'a>() -> impl Parser<'a, &'a str, Vec<char>, extra::Err<Rich<'a, char>>> + Clone {
    one_of("0123456789").repeated().at_least(1).collect()
}

#[inline]
fn error_map<'a, T, E, I>(result: Result<T, E>, span: SimpleSpan) -> Result<T, Rich<'a, I>>
where
    E: ToString,
    I: std::hash::Hash + Eq,
{
    result.map_err(|err| Rich::custom(span, err))
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;

    #[test]
    fn number_digits() {
        println!(
            "{:?}",
            super::number_digits()
                .parse("123")
                .errors()
                .collect::<Vec<_>>()
        );
        assert_eq!(
            super::number_digits().parse("123").output(),
            Some(&vec!['1', '2', '3'])
        );
        assert!(super::number_digits().parse("abc").has_errors());
    }

    #[test]
    fn number_int() {
        assert_eq!(super::number_int().parse("123").output(), Some(&123));
        assert!(super::number_int().parse("abc").has_errors());
    }

    #[test]
    fn number_hex() {
        assert_eq!(super::number_hex().parse("0x123").output(), Some(&0x123));
        assert!(super::number_hex().parse("abc").has_errors());
    }

    #[test]
    fn number_float_basic() {
        assert_eq!(
            super::number_float_basic().parse("123.0").output(),
            Some(&123.0)
        );
        assert_eq!(
            super::number_float_basic().parse("123.456").output(),
            Some(&123.456)
        );
        assert!(super::number_float_basic().parse("abc").has_errors());
    }

    #[test]
    fn number_float_exponent() {
        assert_eq!(
            super::number_float_exponent().parse("123.456e-2").output(),
            Some(&1.23456)
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456e+2").output(),
            Some(&12345.6)
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456e2").output(),
            Some(&12345.6)
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456E-2").output(),
            Some(&1.23456)
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456E+2").output(),
            Some(&12345.6)
        );
        assert_eq!(
            super::number_float_exponent().parse("123.456E2").output(),
            Some(&12345.6)
        );
        assert!(super::number_float_exponent().parse("abc").has_errors());
    }
}
