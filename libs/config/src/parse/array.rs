use hemtt_tokens::{whitespace, Symbol, Token};
use peekmore::PeekMoreIterator;

use crate::{Array, Entry, Error};

use super::{Options, Parse};

impl Parse for Array {
    fn parse(
        options: &Options,
        tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        from: &Token,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Some(token) = tokens.next() {
            if token.symbol() != &Symbol::LeftBrace {
                return Err(Error::UnexpectedToken {
                    token: Box::new(token),
                    expected: vec![Symbol::LeftBrace],
                });
            }
        } else {
            return Err(Error::UnexpectedEOF {
                token: Box::new(from.clone()),
            });
        }
        let mut elements = Vec::new();
        let mut first = true;
        loop {
            let skipped = whitespace::skip_newline(tokens);
            let last = skipped.last().cloned();
            if let Some(token) = tokens.peek() {
                if token.symbol() == &Symbol::RightBrace {
                    if first || options.array_allow_trailing_comma() {
                        tokens.next();
                        break;
                    }
                    return Err(Error::UnexpectedToken {
                        token: Box::new(tokens.next().unwrap()),
                        expected: vec![Symbol::LeftBrace, Symbol::DoubleQuote, Symbol::Digit(0)],
                    });
                }
            } else {
                return Err(Error::UnexpectedEOF {
                    token: Box::new(last.unwrap_or_else(|| from.clone())),
                });
            }
            let entry = Entry::parse(options, tokens, from)?;
            elements.push(entry);
            first = false;
            let skipped = whitespace::skip_newline(tokens);
            let last = skipped.last().cloned();
            if let Some(token) = tokens.next() {
                if token.symbol() == &Symbol::RightBrace {
                    break;
                } else if token.symbol() != &Symbol::Comma {
                    return Err(Error::UnexpectedToken {
                        token: Box::new(token),
                        expected: vec![Symbol::Comma, Symbol::RightBrace],
                    });
                }
            } else {
                return Err(Error::UnexpectedEOF {
                    token: Box::new(last.unwrap_or_else(|| from.clone())),
                });
            }
        }
        whitespace::skip_newline(tokens);
        Ok(Self {
            expand: false,
            elements,
        })
    }
}
