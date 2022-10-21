use std::iter::Peekable;

use super::Token;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Whitespace {
    Space,
    Tab,
}

impl ToString for Whitespace {
    fn to_string(&self) -> String {
        match self {
            Self::Space => " ",
            Self::Tab => "\t",
        }
        .to_string()
    }
}

pub fn skip(input: &mut Peekable<impl Iterator<Item = Token>>) {
    while let Some(token) = input.peek() {
        if token.symbol().is_whitespace() {
            input.next();
        } else {
            break;
        }
    }
}

pub fn skip_newline(input: &mut Peekable<impl Iterator<Item = Token>>) {
    while let Some(token) = input.peek() {
        if token.symbol().is_whitespace() || token.symbol().is_newline() {
            input.next();
        } else {
            break;
        }
    }
}
