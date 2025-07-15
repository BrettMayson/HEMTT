//! HEMTT - Arma 3 P3D Reader

// Parts of the following code is derivative work of the code from the armake2 project by KoffeinFlummi,
// which is licensed GPLv2. This code therefore is also licensed under the terms
// of the GNU Public License, verison 2.

// The original code can be found here:
// https://github.com/KoffeinFlummi/armake2/blob/4b736afc8c615cf49a0d1adce8f6b9a8ae31d90f/src/p3d.rs

use std::io::{BufReader, BufWriter, Read, Seek, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

mod error;
mod face;
mod functions;
mod lod;
mod point;
mod vertex;

pub use error::Error;
pub use face::Face;
pub use functions::*;
pub use lod::LOD;
pub use point::Point;
pub use vertex::Vertex;

#[derive(Debug, Serialize)]
pub struct P3D {
    pub version: u32,
    pub lods: Vec<LOD>,
}

impl P3D {
    /// Reads a P3D from a given input stream.
    ///
    /// # Errors
    /// [`std::io::Error`] if an IO error occurs.
    pub fn read<I: Read + Seek>(input: &mut I) -> Result<Self, Error> {
        let mut reader = BufReader::new(input);

        let mut buffer = [0; 4];
        reader.read_exact(&mut buffer)?;
        if &buffer != b"MLOD" {
            return Err(Error::UnsupportedP3DType(
                String::from_utf8_lossy(&buffer).to_string(),
            ));
        }

        let version = reader.read_u32::<LittleEndian>()?;
        let num_lods = reader.read_u32::<LittleEndian>()?;
        let mut lods: Vec<LOD> = Vec::with_capacity(num_lods as usize);

        for _i in 0..num_lods {
            lods.push(LOD::read(&mut reader)?);
        }

        Ok(Self { version, lods })
    }

    /// Writes the P3D to a given output stream.
    ///
    /// # Errors
    /// [`std::io::Error`] if an IO error occurs.
    /// [`Error::ExceededMaxLodCount`] if the number of LODs exceeds the maximum value.
    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), Error> {
        let mut writer = BufWriter::new(output);

        let lods_len = u32::try_from(self.lods.len())
            .map_err(|_| Error::ExceededMaxLodCount(self.lods.len() as u64))?;

        writer.write_all(b"MLOD")?;
        writer.write_u32::<LittleEndian>(self.version)?;
        writer.write_u32::<LittleEndian>(lods_len)?;

        for lod in &self.lods {
            lod.write(&mut writer)?;
        }

        Ok(())
    }

    #[must_use]
    pub fn dependencies(&self) -> Vec<String> {
        let mut dependencies = Vec::new();
        for lod in &self.lods {
            for face in &lod.faces {
                if !face.texture.is_empty()
                    && !dependencies.contains(&face.texture)
                    && !face.texture.starts_with('#')
                {
                    dependencies.push(face.texture.clone());
                }
                if !face.material.is_empty()
                    && !dependencies.contains(&face.material)
                    && !face.material.starts_with('#')
                {
                    dependencies.push(face.material.clone());
                }
            }
        }
        dependencies
    }
}
