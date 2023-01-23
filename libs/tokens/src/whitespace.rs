//! Whitespace and comments

use peekmore::PeekMoreIterator;

use crate::symbol::Symbol;

use super::Token;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Whitespace characters
pub enum Whitespace {
    /// A space
    Space,
    /// A tab (\t)
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

/// Skip through whitespace
pub fn skip(input: &mut PeekMoreIterator<impl Iterator<Item = Token>>) -> Option<Token> {
    let mut last = None;
    while let Some(token) = input.peek() {
        if token.symbol().is_whitespace() {
            last = input.next();
        } else if token.symbol() == &Symbol::Slash {
            if let Some(next_token) = input.peek_forward(1) {
                if next_token.symbol() == &Symbol::Slash {
                    last = skip_comment(input);
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    last
}

/// Skip through whitespace and newlines
pub fn skip_newline(input: &mut PeekMoreIterator<impl Iterator<Item = Token>>) -> Option<Token> {
    let mut last = None;
    while let Some(token) = input.peek() {
        if token.symbol().is_whitespace() || token.symbol().is_newline() {
            last = input.next();
        } else if token.symbol() == &Symbol::Slash {
            if let Some(next_token) = input.peek_forward(1) {
                if next_token.symbol() == &Symbol::Slash {
                    last = skip_comment(input);
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    last
}

/// Skip through a comment until a newline is found
/// Assumes the slashes are peeked but not consumed
pub fn skip_comment(input: &mut PeekMoreIterator<impl Iterator<Item = Token>>) -> Option<Token> {
    input.next();
    let mut last = input.next();
    while let Some(token) = input.peek() {
        if token.symbol() == &Symbol::Newline {
            break;
        }
        last = input.next();
    }
    last
}
