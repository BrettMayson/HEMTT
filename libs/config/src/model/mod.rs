mod array;
mod class;
mod entry;
mod ident;
mod number;
mod str;

use std::io::Cursor;

use byteorder::{LittleEndian, WriteBytesExt};
use hemtt_tokens::Token;
use peekmore::PeekMoreIterator;

use crate::{error::Error, Options, Rapify};

use self::class::Properties;
pub use self::str::Str;
pub use array::Array;
pub use class::{Children, Class};
pub use entry::Entry;
pub use ident::Ident;
pub use number::Number;

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

#[derive(Debug)]
/// A config file
pub struct Config {
    /// The root, unnamed class
    pub root: Class,
}

impl Parse for Config {
    fn parse(
        options: &Options,
        tokens: &mut PeekMoreIterator<impl Iterator<Item = Token>>,
        from: &Token,
    ) -> Result<Self, Error> {
        let properties: Properties = Properties::parse(options, tokens, from)?;
        Ok(Self {
            root: Class::Local {
                children: Children(properties),
                name: Ident::default(),
                parent: None,
            },
        })
    }
}

impl Rapify for Config {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        _offset: usize,
    ) -> Result<usize, std::io::Error> {
        output.write_all(b"\0raP")?;
        output.write_all(b"\0\0\0\0\x08\0\0\0")?;

        let buffer: Box<[u8]> = vec![0; self.root.rapified_length()].into_boxed_slice();
        let mut cursor = Cursor::new(buffer);
        let written = self.root.rapify(&mut cursor, 16)?;
        // assert_eq!(written, self.root.rapified_length());

        let enum_offset = 16 + cursor.get_ref().len() as u32;
        output.write_u32::<LittleEndian>(enum_offset)?;

        output.write_all(cursor.get_ref())?;

        output.write_all(b"\0\0\0\0")?;
        Ok(written + 4)
    }

    fn rapified_length(&self) -> usize {
        20 + self.root.rapified_length()
    }
}
