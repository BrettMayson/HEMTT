use hemtt_tokens::{Symbol, Token};
use peekmore::PeekMoreIterator;

use crate::{Error, Ident};

use super::{Options, Parse};

impl Parse for Ident {
    fn parse(
        _options: &Options,
        tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        _from: &Token,
    ) -> Result<Self, crate::error::Error>
    where
        Self: Sized,
    {
        let mut ident = Vec::new();
        while let Some(token) = tokens.peek() {
            match token.symbol() {
                Symbol::Digit(_) | Symbol::Word(_) => {
                    ident.push(tokens.next().unwrap());
                }
                Symbol::Join => {
                    tokens.next();
                }
                _ => break,
            }
        }
        if ident.is_empty() {
            return Err(Error::ExpectedIdent {
                token: Box::new(tokens.peek().unwrap().clone()),
            });
        }
        Ok(Self::new(ident))
    }
}
