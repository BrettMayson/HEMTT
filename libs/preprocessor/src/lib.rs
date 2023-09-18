#![deny(clippy::all, clippy::nursery, missing_docs)]
#![warn(clippy::pedantic)]

//! HEMTT - Arma 3 Preprocessor

mod codes;
mod defines;
mod definition;
mod error;
mod ifstate;
mod parse;
mod processor;

pub use error::Error;
pub use processor::Processor;
