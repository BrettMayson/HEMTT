use hemtt_tokens::{Symbol, Token};
use peekmore::PeekMoreIterator;

use crate::{
    error::Error,
    rapify::{Rapify, WriteExt},
    Options,
};

use super::Parse;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A string value
pub struct Str(pub String);

impl Parse for Str {
    fn parse(
        _options: &Options,
        tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        from: &Token,
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
            return Err(Error::UnexpectedEOF {
                token: Box::new(from.clone()),
            });
        }
        let mut string = String::new();
        'outer: loop {
            if let Some(token) = tokens.peek() {
                match token.symbol() {
                    Symbol::DoubleQuote => 'inner: loop {
                        tokens.next();
                        if let Some(token) = tokens.peek() {
                            match token.symbol() {
                                Symbol::DoubleQuote => {
                                    tokens.next();
                                    string.push('"');
                                    break 'inner;
                                }
                                Symbol::Whitespace(_) => continue,
                                Symbol::Escape => {
                                    if tokens.peek_nth(1).unwrap().symbol()
                                        == &Symbol::Word(String::from("n"))
                                    {
                                        tokens.next();
                                        tokens.next();
                                        string.push('\n');
                                        loop {
                                            if let Some(token) = tokens.peek() {
                                                match token.symbol() {
                                                    Symbol::Whitespace(_) => {
                                                        tokens.next();
                                                        continue;
                                                    }
                                                    Symbol::DoubleQuote => {
                                                        tokens.next();
                                                        break 'inner;
                                                    }
                                                    _ => break 'outer,
                                                }
                                            }
                                            return Err(Error::UnexpectedEOF {
                                                token: Box::new(from.clone()),
                                            });
                                        }
                                    }
                                    break;
                                }
                                _ => break 'outer,
                            }
                        }
                        break 'outer;
                    },
                    _ => {
                        string.push_str(&tokens.next().unwrap().to_string());
                    }
                }
            } else {
                return Err(Error::UnexpectedEOF {
                    token: Box::new(from.clone()),
                });
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
    use hemtt_tokens::Token;
    use peekmore::PeekMore;

    use crate::model::Parse;

    #[test]
    fn string() {
        let mut tokens = hemtt_preprocessor::preprocess_string(r#""test""#)
            .unwrap()
            .into_iter()
            .peekmore();
        let string = super::Str::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(string, super::Str("test".to_string()));
    }

    #[test]
    fn string_escape() {
        let mut tokens = hemtt_preprocessor::preprocess_string(r#""test is ""cool""""#)
            .unwrap()
            .into_iter()
            .peekmore();
        let string = super::Str::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(string, super::Str(r#"test is "cool""#.to_string()));
    }

    #[test]
    // fn who_in_the_f_thought_this_was_a_good_idea() {
    fn multiline_string() {
        let mut tokens = hemtt_preprocessor::preprocess_string(r#""test" \n "is" \n "cool""#)
            .unwrap()
            .into_iter()
            .peekmore();
        let string = super::Str::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(string, super::Str("test\nis\ncool".to_string()));
    }
}
