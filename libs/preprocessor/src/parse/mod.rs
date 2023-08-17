use std::path::PathBuf;

use hemtt_common::{
    position::{LineCol, Position},
    workspace::WorkspacePath,
};

use pest::Parser;
use pest_derive::Parser;

use crate::{symbol::Symbol, token::Token, whitespace::Whitespace, Error};

#[derive(Parser)]
#[grammar = "parse/config.pest"]
pub struct PreprocessorParser;

/// Parse a file into tokens
///
/// # Errors
/// If the file is invalid
///
/// # Panics
/// If the file is invalid
pub fn parse(path: &WorkspacePath) -> Result<Vec<Token>, Error> {
    let source = path.read_to_string()?;
    let pairs = PreprocessorParser::parse(Rule::file, &source)?;
    let mut tokens = Vec::new();
    let mut line = 1;
    let mut col = 1;
    let mut offset = 0;
    for pair in pairs {
        let start = LineCol(offset, (line, col));
        match pair.as_rule() {
            Rule::newline => {
                line += 1;
                col = 1;
            }
            Rule::COMMENT => {
                let lines = pair.as_str().split('\n').collect::<Vec<_>>();
                let count = lines.len() - 1;
                line += count;
                if count > 0 {
                    col = lines.last().unwrap().len() + 1;
                } else {
                    col = 1;
                }
                if pair.as_str().starts_with("//") {
                    tokens.push(Token::new(
                        Symbol::Newline,
                        Position::new(
                            LineCol(offset + pair.as_str().len(), (line, col)),
                            LineCol(offset + pair.as_str().len() + 1, (line, col + 1)),
                            PathBuf::from(path.as_str()),
                        ),
                    ));
                }
            }
            _ => {
                col += pair.as_str().len();
            }
        }
        offset += pair.as_str().len();
        let end = LineCol(offset, (line, col));
        tokens.push(Token::new(
            Symbol::to_symbol(pair),
            Position::new(start, end, PathBuf::from(path.as_str())),
        ));
    }
    Ok(tokens)
}

trait Parse {
    fn to_symbol(pair: pest::iterators::Pair<Rule>) -> Self;
}

#[allow(clippy::fallible_impl_from)] // TODO
impl Parse for Symbol {
    fn to_symbol(pair: pest::iterators::Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::word => Self::from_word(pair.as_str().to_string()),
            Rule::alpha => Self::Alpha(pair.as_str().chars().next().unwrap()),
            Rule::digit => Self::Digit(pair.as_str().parse::<usize>().unwrap()),
            Rule::underscore => Self::Underscore,
            Rule::left_parentheses => Self::LeftParenthesis,
            Rule::right_parentheses => Self::RightParenthesis,
            Rule::join => Self::Join,
            Rule::directive => Self::Directive,
            Rule::escape => Self::Escape,
            Rule::comma => Self::Comma,
            Rule::double_quote => Self::DoubleQuote,
            Rule::single_quote => Self::SingleQuote,
            Rule::left_angle => Self::LeftAngle,
            Rule::right_angle => Self::RightAngle,
            Rule::equals => Self::Equals,

            Rule::unicode => Self::Unicode(pair.as_str().to_string()),

            Rule::newline => Self::Newline,
            Rule::space => Self::Whitespace(Whitespace::Space),
            Rule::tab => Self::Whitespace(Whitespace::Tab),
            Rule::WHITESPACE => Self::to_symbol(pair.into_inner().next().unwrap()),
            Rule::COMMENT => Self::Comment(pair.as_str().to_string()),
            Rule::EOI | Rule::file => Self::Eoi,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple() {
        let workspace = hemtt_common::workspace::Workspace::builder()
            .memory()
            .finish()
            .unwrap();
        let test = workspace.join("test.hpp").unwrap();
        test.create_file()
            .unwrap()
            .write_all(b"value = 1;")
            .unwrap();
        let tokens = crate::parse::parse(&test).unwrap();
        assert_eq!(tokens.len(), 7);
    }

    #[test]
    fn unicode() {
        let workspace = hemtt_common::workspace::Workspace::builder()
            .memory()
            .finish()
            .unwrap();
        let test = workspace.join("test.hpp").unwrap();
        let content = "² ƒ ‡ Œ Š – µ œ š ˆ ˜ € º ¨ ¬ 🤔";
        test.create_file()
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
        let tokens = crate::parse::parse(&test).unwrap();
        assert_eq!(tokens.len(), content.chars().count() + 1); // +1 for EOI
    }
}
