#![deny(clippy::all, clippy::nursery, missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

//! HEMTT - Arma 3 Config Parser
//!
//! Requires that files first be tokenized by the [`hemtt_preprocessor`] crate.

mod error;
mod model;
use chumsky::{prelude::Simple, Parser};
pub use model::*;
pub mod parse;
pub mod rapify;

pub use error::Error;

/// Parse a config file
///
/// # Errors
/// If the file is invalid
pub fn parse(input: &str) -> Result<Config, Vec<Simple<char>>> {
    parse::config().parse(input)
}
