use std::sync::Arc;

use chumsky::prelude::*;

use crate::{Ident, parse::ParseError};

pub fn ident<'src>() -> impl Parser<'src, &'src str, Spanned<Ident>, ParseError<'src>> + Clone {
    one_of("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_")
        .repeated()
        .at_least(1)
        .to_slice()
        .map(|value| Ident(Arc::from(value)))
        .spanned()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chumsky::Parser;

    use crate::Ident;

    #[test]
    fn ident() {
        assert_eq!(
            super::ident().parse("abc").into_output().map(|s| s.inner),
            Some(Ident(Arc::from("abc")))
        );
        assert_eq!(
            super::ident()
                .parse("abc123")
                .into_output()
                .map(|s| s.inner),
            Some(Ident(Arc::from("abc123")))
        );
        assert_eq!(
            super::ident()
                .parse("abc_123")
                .into_output()
                .map(|s| s.inner),
            Some(Ident(Arc::from("abc_123")))
        );
    }
}
