use hemtt_tokens::{symbol::Symbol, Token};

use crate::{error::Error, Options, Parse};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ident(Vec<Token>);

impl Parse for Ident {
    fn parse(
        _options: &Options,
        tokens: &mut std::iter::Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Self, crate::error::Error>
    where
        Self: Sized,
    {
        let mut ident = Vec::new();
        while let Some(token) = tokens.peek() {
            match token.symbol() {
                Symbol::Word(_) => {
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
        Ok(Self(ident))
    }
}

impl ToString for Ident {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<String>()
    }
}
