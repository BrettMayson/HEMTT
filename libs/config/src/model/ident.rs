use hemtt_tokens::{Symbol, Token};
use peekmore::PeekMoreIterator;

use crate::{error::Error, Options, Parse};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// An identifier
///
/// ```cpp
/// my_ident = 1;
/// ```
///
/// ```cpp
/// class my_ident {
///    ...
/// };
/// ```
pub struct Ident(Vec<Token>);

impl Parse for Ident {
    fn parse(
        _options: &Options,
        tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
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
