use std::iter::Sum;

use hemtt_tokens::{whitespace, Symbol, Token};
use peekmore::PeekMoreIterator;

use crate::{
    error::Error,
    rapify::{compressed_int_len, Rapify, WriteExt},
    Options,
};

use super::{Entry, Parse};

#[derive(Debug, Clone, PartialEq)]
/// An array of entries
pub struct Array {
    /// Is the array expanding a previously defined array
    ///
    /// ```cpp
    /// my_array[] += {1,2,3};
    /// ```
    pub expand: bool,
    /// The elements of the array
    pub elements: Vec<Entry>,
}

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

impl Rapify for Array {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        offset: usize,
    ) -> Result<usize, std::io::Error> {
        let mut written = output.write_compressed_int(self.elements.len() as u32)?;

        // if self.expand {
        //     output.write_all(&[1, 0, 0, 0])?;
        //     written += 4;
        // }

        for element in &self.elements {
            if let Some(code) = element.rapified_code() {
                output.write_all(&[code])?;
                written += 1;
            }
            written += element.rapify(output, offset).unwrap();
        }

        assert_eq!(written, self.rapified_length());

        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        let len = compressed_int_len(self.elements.len() as u32)
            + usize::sum(self.elements.iter().map(|e| e.rapified_length() + 1));
        // if self.expand {
        //     len + 4
        // } else {
        //     len
        // }
        len
    }
}
