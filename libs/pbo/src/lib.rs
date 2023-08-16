#![deny(clippy::all, clippy::nursery, missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation, clippy::cast_lossless)]

//! HEMTT - Arma 3 PBO Writer/Reader

// Parts of the following code is derivative work of the code from the armake2 project by KoffeinFlummi,
// which is licensed GPLv2. This code therefore is also licensed under the terms
// of the GNU Public License, verison 2.

// The original code can be found here:
// https://github.com/KoffeinFlummi/armake2/blob/4b736afc8c615cf49a0d1adce8f6b9a8ae31d90f/src/pbo.rs

use std::io::{Read, Write};

mod error;
mod file;
mod model;
mod read;
mod sign_version;
mod write;

pub use error::Error;
pub use model::{Checksum, Header, Mime};
pub use read::ReadablePbo;
pub use sign_version::BISignVersion;
pub use write::WritablePbo;

trait WritePbo {
    fn write_pbo<O: Write>(&self, output: &mut O) -> Result<(), Error>;
}

trait ReadPbo: Sized {
    fn read_pbo<I: Read>(input: &mut I) -> Result<(Self, usize), crate::error::Error>;
}
