use hemtt_tokens::symbol::Symbol;

use crate::{
    error::Error,
    rapify::{Rapify, WriteExt},
    Options,
};

use super::Parse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Str(pub String);

impl Parse for Str {
    fn parse(
        _options: &Options,
        tokens: &mut std::iter::Peekable<impl Iterator<Item = hemtt_tokens::Token>>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Some(token) = tokens.next() {
            if token.symbol() != &Symbol::DoubleQuote {
                return Err(Error::UnexpectedToken {
                    token: Box::new(token),
                    expected: vec![Symbol::DoubleQuote],
                });
            }
        } else {
            return Err(Error::UnexpectedEOF);
        }
        let mut string = String::new();
        loop {
            if let Some(token) = tokens.peek() {
                match token.symbol() {
                    Symbol::DoubleQuote => {
                        tokens.next();
                        if let Some(token) = tokens.peek() {
                            if token.symbol() == &Symbol::DoubleQuote {
                                tokens.next();
                                string.push('"');
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    Symbol::Newline => {
                        return Err(Error::UnexpectedToken {
                            token: Box::new(token.clone()),
                            expected: vec![Symbol::DoubleQuote],
                        });
                    }
                    _ => {
                        string.push_str(&tokens.next().unwrap().to_string());
                    }
                }
            } else {
                return Err(Error::UnexpectedEOF);
            }
        }
        Ok(Self(string))
    }
}

impl Rapify for Str {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        _offset: usize,
    ) -> Result<usize, std::io::Error> {
        output.write_cstring(&self.0)?;
        Ok(self.0.len() + 1)
    }

    fn rapified_length(&self) -> usize {
        self.0.len() + 1
    }

    fn rapified_code(&self) -> Option<u8> {
        Some(0)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Parse;

    #[test]
    fn string() {
        let mut tokens = hemtt_preprocessor::preprocess_string(r#""test""#)
            .unwrap()
            .into_iter()
            .peekable();
        let string = super::Str::parse(&crate::Options::default(), &mut tokens).unwrap();
        assert_eq!(string, super::Str("test".to_string()));
    }

    #[test]
    fn string_escape() {
        let mut tokens = hemtt_preprocessor::preprocess_string(r#""test is ""cool""""#)
            .unwrap()
            .into_iter()
            .peekable();
        let string = super::Str::parse(&crate::Options::default(), &mut tokens).unwrap();
        assert_eq!(string, super::Str(r#"test is "cool""#.to_string()));
    }
}
