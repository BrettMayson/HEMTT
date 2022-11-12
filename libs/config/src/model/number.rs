use byteorder::{LittleEndian, WriteBytesExt};
use hemtt_tokens::symbol::Symbol;
use peekmore::PeekMoreIterator;

use crate::{error::Error, rapify::Rapify, Options};

use super::Parse;

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Int32(i32),
    Int64(i64),
    Float32(f32),
}

impl Parse for Number {
    fn parse(
        _options: &Options,
        tokens: &mut PeekMoreIterator<impl Iterator<Item = hemtt_tokens::Token>>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut buffer: i64 = 0;
        let mut negative = false;
        let mut seen_digit = false;
        while let Some(token) = tokens.peek() {
            match token.symbol() {
                Symbol::Dash => {
                    if seen_digit || negative {
                        return Err(Error::UnexpectedToken {
                            token: Box::new(token.clone()),
                            expected: vec![Symbol::Decimal],
                        });
                    }
                    tokens.next();
                    negative = true;
                }
                Symbol::Digit(digit) => {
                    buffer = buffer * 10 + *digit as i64;
                    tokens.next();
                    seen_digit = true;
                }
                Symbol::Decimal => {
                    if !seen_digit {
                        return Err(Error::UnexpectedToken {
                            token: Box::new(token.clone()),
                            expected: vec![Symbol::Decimal],
                        });
                    }
                    let mut decimal = 0;
                    let mut decimal_place = 0;
                    let mut current_token = tokens.next().unwrap();
                    while let Some(token) = tokens.peek() {
                        match token.symbol() {
                            Symbol::Digit(digit) => {
                                decimal = decimal * 10 + *digit as i64;
                                decimal_place += 1;
                                current_token = tokens.next().unwrap();
                            }
                            _ => break,
                        }
                    }
                    if decimal_place == 0 {
                        return Err(Error::UnexpectedToken {
                            token: Box::new(current_token),
                            expected: vec![Symbol::Digit(0)],
                        });
                    }
                    #[allow(clippy::cast_precision_loss)]
                    return Ok(Self::Float32(
                        buffer as f32 + decimal as f32 / 10f32.powi(decimal_place),
                    ));
                }
                _ => break,
            }
        }
        if negative {
            buffer = -buffer;
        }
        if buffer > i64::from(i32::MAX) {
            Ok(Self::Int64(buffer))
        } else {
            #[allow(clippy::cast_possible_truncation)]
            Ok(Self::Int32(buffer as i32))
        }
    }
}

impl Rapify for Number {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        _offset: usize,
    ) -> Result<usize, std::io::Error> {
        let mut written = 0;
        match self {
            Self::Int32(i) => {
                output.write_i32::<LittleEndian>(*i)?;
                written += 4;
            }
            Self::Int64(i) => {
                output.write_i64::<LittleEndian>(*i)?;
                written += 8;
            }
            Self::Float32(f) => {
                output.write_f32::<LittleEndian>(*f)?;
                written += 4;
            }
        }
        Ok(written)
    }

    fn rapified_length(&self) -> usize {
        match self {
            Self::Int32(_) | Self::Float32(_) => 4,
            Self::Int64(_) => 8,
        }
    }

    fn rapified_code(&self) -> Option<u8> {
        match self {
            Self::Int32(_) => Some(2),
            Self::Int64(_) => Some(6),
            Self::Float32(_) => Some(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use peekmore::PeekMore;

    use crate::model::Parse;

    #[test]
    fn i64() {
        let mut tokens = hemtt_preprocessor::preprocess_string("12345678901234567")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(&crate::Options::default(), &mut tokens).unwrap();
        assert_eq!(number, super::Number::Int64(12_345_678_901_234_567));
    }

    #[test]
    fn i32() {
        let mut tokens = hemtt_preprocessor::preprocess_string("1234567890")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(&crate::Options::default(), &mut tokens).unwrap();
        assert_eq!(number, super::Number::Int32(1_234_567_890));
    }

    #[test]
    fn f32() {
        let mut tokens = hemtt_preprocessor::preprocess_string("1234567890.1234567890")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(&crate::Options::default(), &mut tokens).unwrap();
        assert_eq!(number, super::Number::Float32(1_234_567_890.123_456_789));
    }
}
