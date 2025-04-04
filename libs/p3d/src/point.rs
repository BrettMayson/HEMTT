use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use crate::Error;

#[derive(Debug, Default, PartialEq, Serialize)]
pub struct Point {
    pub coords: (f32, f32, f32),
    pub flags: u32,
}

impl Point {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            coords: (0.0, 0.0, 0.0),
            flags: 0,
        }
    }

    /// Reads a Point from a given input stream.
    ///
    /// # Errors
    /// [`std::io::Error`] if an IO error occurs.
    pub fn read<I: Read>(input: &mut I) -> Result<Self, Error> {
        Ok(Self {
            coords: (
                input.read_f32::<LittleEndian>()?,
                input.read_f32::<LittleEndian>()?,
                input.read_f32::<LittleEndian>()?,
            ),
            flags: input.read_u32::<LittleEndian>()?,
        })
    }

    /// Writes the Point to a given output stream.
    ///
    /// # Errors
    /// [`std::io::Error`] if an IO error occurs.
    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), Error> {
        output.write_f32::<LittleEndian>(self.coords.0)?;
        output.write_f32::<LittleEndian>(self.coords.1)?;
        output.write_f32::<LittleEndian>(self.coords.2)?;
        output.write_u32::<LittleEndian>(self.flags)?;
        Ok(())
    }
}
