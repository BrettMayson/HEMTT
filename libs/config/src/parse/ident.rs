use chumsky::prelude::*;

use crate::Ident;

pub fn ident<'a>() -> impl Parser<'a, &'a str, Ident, extra::Err<Rich<'a, char>>> + Clone {
    one_of("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map_with(|value: String, extra| Ident {
            value,
            span: (extra.span() as SimpleSpan).into_range(),
        })
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;

    use crate::Ident;

    #[test]
    fn ident() {
        assert_eq!(
            super::ident().parse("abc").output(),
            Some(&Ident {
                value: "abc".to_string(),
                span: 0..3,
            })
        );
        assert_eq!(
            super::ident().parse("abc123").output(),
            Some(&Ident {
                value: "abc123".to_string(),
                span: 0..6,
            })
        );
        assert_eq!(
            super::ident().parse("abc_123").output(),
            Some(&Ident {
                value: "abc_123".to_string(),
                span: 0..7,
            })
        );
    }
}
