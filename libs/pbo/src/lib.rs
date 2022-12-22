#![deny(clippy::all, clippy::nursery, missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation, clippy::cast_lossless)]

//! HEMTT - Arma 3 PBO Writer/Reader

use std::io::{Read, Write};

mod error;
mod file;
mod model;
pub mod prefix;
mod read;
mod sign_version;
mod write;

pub use error::Error;
pub use model::{Checksum, Header, Mime};
pub use prefix::Prefix;
pub use read::ReadablePbo;
pub use sign_version::BISignVersion;
pub use write::WritablePbo;

trait WritePbo {
    fn write_pbo<O: Write>(&self, output: &mut O) -> Result<(), Error>;
}

trait ReadPbo: Sized {
    fn read_pbo<I: Read>(input: &mut I) -> Result<(Self, usize), crate::error::Error>;
}
