use hemtt_tokens::{position::Position, symbol::Symbol, whitespace::Whitespace, Token};
use pest::Parser;
use pest_derive::Parser;

use crate::Error;

#[derive(Parser)]
#[grammar = "parse/config.pest"]
pub struct PreprocessorParser;

pub fn parse(path: &str, source: &str) -> Result<Vec<Token>, Error> {
    let pairs = PreprocessorParser::parse(Rule::file, source)?;
    let mut tokens = Vec::new();
    let mut line = 1;
    let mut col = 1;
    let mut offset = 0;
    for pair in pairs {
        let start = (offset, (line, col));
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
                            (offset + pair.as_str().len(), (line, col)),
                            (offset + pair.as_str().len() + 1, (line, col + 1)),
                            path.to_string(),
                        ),
                    ));
                }
            }
            _ => {
                col += pair.as_str().len();
            }
        }
        offset += pair.as_str().len();
        let end = (offset, (line, col));
        tokens.push(Token::new(
            Symbol::to_symbol(pair),
            Position::new(start, end, path.to_string()),
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
            Rule::dash => Self::Dash,
            Rule::assignment => Self::Assignment,
            Rule::plus => Self::Plus,
            Rule::left_brace => Self::LeftBrace,
            Rule::right_brace => Self::RightBrace,
            Rule::left_bracket => Self::LeftBracket,
            Rule::right_bracket => Self::RightBracket,
            Rule::left_parentheses => Self::LeftParenthesis,
            Rule::right_parentheses => Self::RightParenthesis,
            Rule::colon => Self::Colon,
            Rule::semicolon => Self::Semicolon,
            Rule::join => Self::Join,
            Rule::directive => Self::Directive,
            Rule::escape => Self::Escape,
            Rule::comma => Self::Comma,
            Rule::decimal => Self::Decimal,
            Rule::double_quote => Self::DoubleQuote,
            Rule::single_quote => Self::SingleQuote,
            Rule::left_angle => Self::LeftAngle,
            Rule::right_angle => Self::RightAngle,

            Rule::unicode => Self::Unicode(pair.as_str().to_string()),

            Rule::newline => Self::Newline,
            Rule::space => Self::Whitespace(Whitespace::Space),
            Rule::tab => Self::Whitespace(Whitespace::Tab),
            Rule::WHITESPACE => Self::to_symbol(pair.into_inner().next().unwrap()),
            Rule::COMMENT => Self::Comment(pair.as_str().to_string()),
            Rule::EOI => Self::Eoi,

            Rule::file => Self::Void,
        }
    }
}
