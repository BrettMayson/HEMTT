use chumsky::prelude::*;

use crate::Ident;

pub fn ident() -> impl Parser<char, Ident, Error = Simple<char>> {
    one_of("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map_with_span(|value, span| Ident { value, span })
}

#[cfg(test)]
mod tests {
    use crate::Ident;

    use super::*;

    #[test]
    fn test_ident() {
        assert_eq!(
            ident().parse("abc"),
            Ok(Ident {
                value: "abc".to_string(),
                span: 0..3,
            })
        );
        assert_eq!(
            ident().parse("abc123"),
            Ok(Ident {
                value: "abc123".to_string(),
                span: 0..6,
            })
        );
        assert_eq!(
            ident().parse("abc_123"),
            Ok(Ident {
                value: "abc_123".to_string(),
                span: 0..7,
            })
        );
    }
}
