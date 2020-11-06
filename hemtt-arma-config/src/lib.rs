use pest::error::Error;
use pest::Parser;

#[macro_use]
extern crate pest_derive;

mod linter;
pub use linter::{InheritanceStyle, LinterOptions};
mod token;
use token::{PreProcessParser, Rule, Token};
mod preprocess;
pub use preprocess::preprocess;
mod render;
pub use render::render;

pub fn tokenize(source: &str) -> Result<Vec<Token>, Error<Rule>> {
    let mut tokens = Vec::new();

    let pairs = PreProcessParser::parse(Rule::file, source)?;
    for pair in pairs {
        tokens.push(Token::from(pair))
    }

    Ok(tokens)
}
