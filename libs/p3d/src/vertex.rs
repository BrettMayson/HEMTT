use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::Error;

#[derive(Debug, Default, PartialEq)]
pub struct Vertex {
    pub point_index: u32,
    pub normal_index: u32,
    pub uv: (f32, f32),
}

impl Vertex {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            point_index: 0,
            normal_index: 0,
            uv: (0.0, 0.0),
        }
    }

    /// Reads a Vertex from a given input stream.
    ///
    /// # Errors
    /// [`std::io::Error`] if an IO error occurs.
    pub fn read<I: Read>(input: &mut I) -> Result<Self, Error> {
        Ok(Self {
            point_index: input.read_u32::<LittleEndian>()?,
            normal_index: input.read_u32::<LittleEndian>()?,
            uv: (
                input.read_f32::<LittleEndian>()?,
                input.read_f32::<LittleEndian>()?,
            ),
        })
    }

    /// Writes the Vertex to a given output stream.
    ///
    /// # Errors
    /// [`std::io::Error`] if an IO error occurs.
    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), Error> {
        output.write_u32::<LittleEndian>(self.point_index)?;
        output.write_u32::<LittleEndian>(self.normal_index)?;
        output.write_f32::<LittleEndian>(self.uv.0)?;
        output.write_f32::<LittleEndian>(self.uv.1)?;
        Ok(())
    }
}
