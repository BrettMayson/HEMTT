//! HEMTT - Arma 3 Preprocessor

pub mod codes {
    automod::dir!(pub "src/codes");
}

mod defines;
mod definition;
mod error;
mod ifstate;
mod parse;
mod processor;

pub use error::Error;
pub use processor::Processor;
