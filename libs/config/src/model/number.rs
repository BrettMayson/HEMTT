use byteorder::{LittleEndian, WriteBytesExt};
use hemtt_tokens::{Symbol, Token};
use peekmore::PeekMoreIterator;

use crate::{error::Error, rapify::Rapify, Options};

use super::Parse;

#[derive(Debug, Clone, PartialEq)]
/// A number value
pub enum Number {
    /// A 32-bit integer
    Int32(i32),
    /// A 64-bit integer
    Int64(i64),
    /// A 32-bit floating point number
    Float32(f32),
}

impl Parse for Number {
    #[allow(clippy::too_many_lines)]
    fn parse(
        _options: &Options,
        tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        _from: &Token,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut buffer: i64 = 0;
        let mut negative = false;
        let mut seen_digit = false;
        let mut decimal = 0;
        let mut decimal_place = 0;
        while let Some(token) = tokens.peek() {
            let token = token.clone();
            match token.symbol() {
                Symbol::Word(word) => {
                    if seen_digit && buffer == 0 {
                        // parse hex
                        if word.starts_with('x') {
                            let hex = word.trim_start_matches('x');
                            buffer = i64::from_str_radix(hex, 16).unwrap();
                            tokens.next();
                            if buffer > i64::from(i32::MAX) {
                                return Ok(Self::Int64(buffer));
                            }
                            #[allow(clippy::cast_possible_truncation)]
                            return Ok(Self::Int32(buffer as i32));
                        }
                    }
                    #[allow(clippy::cast_precision_loss)]
                    let val = buffer as f32 + decimal as f32 / 10f32.powi(decimal_place - 1);
                    if word.to_lowercase() == "e" {
                        // 1e-1 or 1e+1
                        let mut positive = true;
                        tokens.next();
                        if let Some(dash) = tokens.peek() {
                            if dash.symbol() == &Symbol::Dash {
                                positive = false;
                            } else if dash.symbol() != &Symbol::Plus {
                                return Err(Error::UnexpectedToken {
                                    token: Box::new(dash.clone()),
                                    expected: vec![Symbol::Dash, Symbol::Digit(0)],
                                });
                            }
                            tokens.next();
                        }
                        let mut exp = 0;
                        while let Some(digit) = tokens.peek() {
                            if let Symbol::Digit(d) = digit.symbol() {
                                exp = exp * 10 + d;
                                tokens.next();
                            } else {
                                break;
                            }
                        }
                        #[allow(clippy::cast_precision_loss)]
                        if positive {
                            return Ok(Self::Float32(val * 10_f32.powf(exp as f32)));
                        }
                        #[allow(clippy::cast_precision_loss)]
                        return Ok(Self::Float32(val / 10_f32.powf(exp as f32)));
                    } else if word.to_lowercase().starts_with('e') {
                        // 1e1
                        tokens.next();
                        let exp = word
                            .to_lowercase()
                            .trim_start_matches('e')
                            .parse::<u32>()
                            .unwrap();
                        #[allow(clippy::cast_precision_loss)]
                        return Ok(Self::Float32(val * 10_f32.powf(exp as f32)));
                    }
                    return Err(Error::ExpectedNumber {
                        token: Box::new(token.clone()),
                    });
                }
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
                    if decimal_place == 0 {
                        buffer = buffer * 10 + *digit as i64;
                    } else {
                        decimal = decimal * 10 + *digit as i64;
                        decimal_place += 1;
                    }
                    seen_digit = true;
                    tokens.next();
                }
                Symbol::Decimal => {
                    if !seen_digit {
                        return Err(Error::UnexpectedToken {
                            token: Box::new(token.clone()),
                            expected: vec![Symbol::Decimal],
                        });
                    }
                    tokens.next();
                    decimal_place = 1;
                }
                Symbol::Join => {
                    tokens.next();
                }
                _ => break,
            }
        }
        if negative {
            buffer = -buffer;
        }
        if decimal_place > 1 {
            #[allow(clippy::cast_precision_loss)]
            Ok(Self::Float32(
                buffer as f32 + decimal as f32 / 10f32.powi(decimal_place - 1),
            ))
        } else if buffer > i64::from(i32::MAX) {
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
    use hemtt_tokens::Token;
    use peekmore::PeekMore;

    use crate::model::Parse;

    #[test]
    fn i64() {
        let mut tokens = hemtt_preprocessor::preprocess_string("12345678901234567")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Int64(12_345_678_901_234_567));
    }

    #[test]
    fn i32() {
        let mut tokens = hemtt_preprocessor::preprocess_string("1234567890")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Int32(1_234_567_890));
        let mut tokens = hemtt_preprocessor::preprocess_string("-1234567890")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Int32(-1_234_567_890));
    }

    #[test]
    fn f32() {
        let mut tokens = hemtt_preprocessor::preprocess_string("1234567890.1234567890")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Float32(1_234_567_890.123_456_789));
        let mut tokens = hemtt_preprocessor::preprocess_string("-1234567890.1234567890")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Float32(-1_234_567_890.123_456_789));
    }

    #[test]
    fn hex() {
        let mut tokens = hemtt_preprocessor::preprocess_string("0x1234567890")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Int64(0x0012_3456_7890));
    }

    #[test]
    fn e() {
        let mut tokens = hemtt_preprocessor::preprocess_string("1e-3")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Float32(1e-3));
        let mut tokens = hemtt_preprocessor::preprocess_string("1e3")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Float32(1e3));
        let mut tokens = hemtt_preprocessor::preprocess_string("1e-007")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Float32(1e-007));
        let mut tokens = hemtt_preprocessor::preprocess_string("1e007")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Float32(1e007));
        let mut tokens = hemtt_preprocessor::preprocess_string("1E007")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Float32(1e007));
        let mut tokens = hemtt_preprocessor::preprocess_string("1E-007")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Float32(1e-007));
        let mut tokens = hemtt_preprocessor::preprocess_string("1e+007")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Float32(1e+007));
        let mut tokens = hemtt_preprocessor::preprocess_string("2.4e9")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Float32(2.4e9));
    }

    #[test]
    fn join() {
        let mut tokens = hemtt_preprocessor::preprocess_string("1##2")
            .unwrap()
            .into_iter()
            .peekmore();
        let number = super::Number::parse(
            &crate::Options::default(),
            &mut tokens,
            &Token::builtin(None),
        )
        .unwrap();
        assert_eq!(number, super::Number::Int32(12));
    }
}
