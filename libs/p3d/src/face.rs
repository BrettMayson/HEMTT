use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hemtt_common::io::{ReadExt, WriteExt};

use crate::{Error, Vertex};

#[derive(Debug, Default, PartialEq)]
pub struct Face {
    pub vertices: Vec<Vertex>,
    pub flags: u32,
    pub texture: String,
    pub material: String,
}

impl Face {
    #[must_use]
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(4),
            flags: 0,
            texture: String::new(),
            material: String::new(),
        }
    }

    /// Reads a Face from a given input stream.
    ///
    /// # Errors
    /// [`std::io::Error`] if an IO error occurs.
    pub fn read<I: Read>(input: &mut I) -> Result<Self, Error> {
        let num_verts = input.read_u32::<LittleEndian>()?;
        if num_verts != 3 && num_verts != 4 {
            return Err(Error::InvalidFaceVertexCount(num_verts));
        }

        let mut vertices: Vec<Vertex> = Vec::with_capacity(num_verts as usize);
        for _i in 0..num_verts {
            vertices.push(Vertex::read(input)?);
        }

        if num_verts == 3 {
            Vertex::read(input)?;
        }

        let flags = input.read_u32::<LittleEndian>()?;
        let texture = input.read_cstring()?;
        let material = input.read_cstring()?;

        Ok(Self {
            vertices,
            flags,
            texture,
            material,
        })
    }

    /// Writes the Face to a given output stream.
    ///
    /// # Errors
    /// [`std::io::Error`] if an IO error occurs.
    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), Error> {
        let vertices_count = u32::try_from(self.vertices.len())
            .map_err(|_| Error::ExceededMaxVertexCount(self.vertices.len() as u64))?;
        if vertices_count != 3 && vertices_count != 4 {
            return Err(Error::InvalidFaceVertexCount(vertices_count));
        }
        output.write_u32::<LittleEndian>(vertices_count)?;

        for vert in &self.vertices {
            vert.write(output)?;
        }
        if vertices_count == 3 {
            let vert = Vertex::new();
            vert.write(output)?;
        }

        output.write_u32::<LittleEndian>(self.flags)?;
        output.write_cstring(&self.texture)?;
        output.write_cstring(&self.material)?;
        Ok(())
    }
}
