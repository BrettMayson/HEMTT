use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use hemtt_workspace::{
    WorkspacePath,
    position::{LineCol, Position},
    reporting::{Symbol, Token, Whitespace},
};

use pest::Parser;
use pest_derive::Parser;

use crate::{Error, codes::pe24_parsing_failed::ParsingFailed};

#[derive(Parser)]
#[grammar = "parse/config.pest"]
/// Parser for the preprocessor, generated from `config.pest`
pub struct PreprocessorParser;

#[derive(Debug, Clone)]
struct Cache {
    tokens: Arc<DashMap<WorkspacePath, Vec<Arc<Token>>>>,
}

impl Cache {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<Cache> = LazyLock::new(|| Cache {
            tokens: Arc::new(DashMap::new()),
        });
        (*SINGLETON).clone()
    }
}

/// Parse a file into tokens
///
/// # Errors
/// If the file is invalid
///
/// # Panics
/// If the file is invalid
pub fn file(path: &WorkspacePath) -> Result<Vec<Arc<Token>>, Error> {
    let cache = Cache::get();
    if cache.tokens.contains_key(path) {
        return Ok(cache.tokens.get(path).expect("exists").value().clone());
    }
    let source = path.read_to_string()?;
    let res = str(&source, path)?;
    // The LSP manages its own caches, having this enabled would cause the LSP to never see any changes
    // This will make the LSP slower, in the future it could have a way to invalidate the cache (when a file is saved)
    #[cfg(not(feature = "lsp"))]
    {
        let path_str = path.as_str();
        if ["macros", "common", "script", "component"]
            .iter()
            .any(|&x| path_str.contains(x))
        {
            cache.tokens.insert(path.clone(), res.clone());
        }
    }
    Ok(res)
}

/// Parse a string into tokens
///
/// # Errors
/// If the string is invalid1
///
/// # Panics
/// If the string is invalid
pub fn str(source: &str, path: &WorkspacePath) -> Result<Vec<Arc<Token>>, Error> {
    let pairs = PreprocessorParser::parse(Rule::file, source)
        .map_err(|e| ParsingFailed::code(e, path.clone()))?;
    let mut tokens = Vec::new();
    let mut line = 1;
    let mut col = 0;
    let mut offset = 0;
    let mut in_single_string = false;
    let mut in_double_string = false;
    let mut skipping_comment = false;
    for pair in pairs {
        let start = LineCol(offset, (line, col));
        match pair.as_rule() {
            Rule::newline => {
                if skipping_comment {
                    skipping_comment = false;
                }
                line += 1;
                col = 0;
            }
            Rule::COMMENT => {
                if in_single_string || in_double_string {
                    if !skipping_comment {
                        tokens.push(Arc::new(Token::new(
                            Symbol::Word(pair.as_str().to_string()),
                            Position::new(
                                start,
                                LineCol(start.0 + 2, (start.1.0 + 2, start.1.1 + 2)),
                                path.clone(),
                            ),
                        )));
                    }
                } else {
                    let lines = pair.as_str().split('\n').collect::<Vec<_>>();
                    let count = lines.len() - 1;
                    line += count;
                    if count > 0 {
                        col = lines
                            .last()
                            .expect("exists because count is greater than 0")
                            .len()
                            + 1;
                    } else {
                        col = 0;
                    }
                    if pair.as_str() == "//" {
                        // skip to end of line
                        skipping_comment = true;
                    }
                }
            }
            Rule::single_quote => {
                if !skipping_comment && !in_double_string {
                    in_single_string = !in_single_string;
                    col += 1;
                }
            }
            Rule::double_quote => {
                if !skipping_comment && !in_single_string {
                    in_double_string = !in_double_string;
                    col += 1;
                }
            }
            _ => {
                col += pair.as_str().len();
            }
        }
        offset += pair.as_str().len();
        if skipping_comment {
            continue;
        }
        let end = LineCol(offset, (line, col));
        tokens.push(Arc::new(Token::new(
            Symbol::to_symbol(pair),
            Position::new(start, end, path.clone()),
        )));
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
            Rule::alpha => Self::Alpha(
                pair.as_str()
                    .chars()
                    .next()
                    .expect("at least one character should exist"),
            ),
            Rule::digit => Self::Digit(
                pair.as_str()
                    .parse::<usize>()
                    .expect("should be a parseable number"),
            ),
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
            Rule::WHITESPACE => {
                Self::to_symbol(pair.into_inner().next().expect("inner token should exist"))
            }
            Rule::COMMENT => Self::Comment(pair.as_str().to_string()),
            Rule::EOI | Rule::file => Self::Eoi,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    #[test]
    fn simple() {
        let workspace = hemtt_workspace::Workspace::builder()
            .memory()
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .unwrap();
        let test = workspace.join("test.hpp").unwrap();
        test.create_file()
            .unwrap()
            .write_all(b"value = 1;")
            .unwrap();
        let tokens = crate::parse::file(&test).unwrap();
        assert_eq!(tokens.len(), 7);
    }

    #[test]
    fn unicode() {
        let workspace = hemtt_workspace::Workspace::builder()
            .memory()
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .unwrap();
        let test = workspace.join("test.hpp").unwrap();
        let content = "² ƒ ‡ Œ Š – µ œ š ˆ ˜ € º ¨ ¬ 🤔";
        test.create_file()
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
        let tokens = crate::parse::file(&test).unwrap();
        assert_eq!(tokens.len(), content.chars().count() + 1); // +1 for EOI
    }
}
