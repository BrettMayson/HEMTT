//! Parsing of config files

mod entry;
mod ident;
mod number;
mod options;
mod str;
pub use options::{Options, Preset};

use hemtt_tokens::Token;
use peekmore::PeekMoreIterator;

use crate::{Children, Class, Config, Error, Ident, Properties};

mod array;
mod class;

/// A trait for parsing a type from a token stream
pub trait Parse {
    /// # Errors
    /// if the token stream is invalid
    fn parse(
        options: &Options,
        tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        from: &Token,
    ) -> Result<Self, Error>
    where
        Self: Sized;
}

impl Parse for Config {
    fn parse(
        options: &Options,
        tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        from: &Token,
    ) -> Result<Self, Error> {
        let properties = Properties::parse(options, tokens, from)?;
        Ok(Self {
            root: Class::Local {
                children: Children(properties),
                name: Ident::default(),
                parent: None,
            },
        })
    }
}
