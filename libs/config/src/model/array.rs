use std::iter::Sum;

use hemtt_tokens::{symbol::Symbol, whitespace};

use crate::{
    error::Error,
    rapify::{compressed_int_len, Rapify, WriteExt},
};

use super::{Entry, Parse};

#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    pub expand: bool,
    pub elements: Vec<Entry>,
}

impl Parse for Array {
    fn parse(
        tokens: &mut std::iter::Peekable<impl Iterator<Item = hemtt_tokens::Token>>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Some(token) = tokens.next() {
            if token.symbol() != &Symbol::LeftBrace {
                return Err(Error::UnexpectedToken {
                    token,
                    expected: vec![Symbol::LeftBrace],
                });
            }
        } else {
            return Err(Error::UnexpectedEOF);
        }
        let mut elements = Vec::new();
        loop {
            let entry = Entry::parse(tokens)?;
            elements.push(entry);
            whitespace::skip_newline(tokens);
            if let Some(token) = tokens.next() {
                if token.symbol() == &Symbol::RightBrace {
                    break;
                } else if token.symbol() != &Symbol::Comma {
                    return Err(Error::UnexpectedToken {
                        token,
                        expected: vec![Symbol::Comma, Symbol::RightBrace],
                    });
                }
            } else {
                return Err(Error::UnexpectedEOF);
            }
            whitespace::skip_newline(tokens);
        }
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

        for element in &self.elements {
            output.write_all(&[element.rapified_code().unwrap()])?;
            written += element.rapify(output, offset).unwrap() + 1;
        }

        assert_eq!(written, self.rapified_length());

        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        let len = compressed_int_len(self.elements.len() as u32)
            + usize::sum(self.elements.iter().map(|e| e.rapified_length() + 1));
        if self.expand {
            len + 4
        } else {
            len
        }
    }
}
