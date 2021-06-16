#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate log;

mod error;
mod linter;
mod parser;
mod preprocess;
pub mod rapify;
pub mod resolver;
pub mod simplify;

pub use error::ArmaConfigError;
pub use linter::{InheritanceStyle, LinterOptions};
pub use parser::parse;
pub use preprocess::{preprocess, render, tokenize};
