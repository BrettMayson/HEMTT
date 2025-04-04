use std::io::Cursor;

use byteorder::{LittleEndian, WriteBytesExt};

use crate::{Class, Config};

use super::{Derapify, Rapify};

impl Rapify for Config {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        _offset: usize,
    ) -> Result<usize, std::io::Error> {
        output.write_all(b"\0raP")?;
        output.write_all(b"\0\0\0\0\x08\0\0\0")?;

        let root_class = Class::Root {
            properties: self.0.clone(),
        };
        let buffer: Box<[u8]> = vec![0; root_class.rapified_length()].into_boxed_slice();
        let mut cursor = Cursor::new(buffer);
        let written = root_class.rapify(&mut cursor, 16)?;
        assert_eq!(written, root_class.rapified_length());

        let enum_offset = 16 + cursor.get_ref().len() as u32;
        output.write_u32::<LittleEndian>(enum_offset)?;

        output.write_all(cursor.get_ref())?;

        output.write_all(b"\0\0\0\0")?;
        assert_eq!(written + 20, self.rapified_length());
        Ok(written + 20)
    }

    fn rapified_length(&self) -> usize {
        let root_class = Class::Root {
            properties: self.0.clone(),
        };
        root_class.rapified_length() + 20 // metadata
    }
}

impl Derapify for Config {
    fn derapify<I: std::io::Read + std::io::Seek>(input: &mut I) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut buffer = vec![0; 4];
        input.read_exact(&mut buffer)?;
        if &buffer != b"\0raP" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid rapified config",
            ));
        }
        input.seek_relative(8)?;
        // skip enum offset
        input.seek_relative(4)?;

        let root_class = Class::derapify(input, None)?;

        if let Class::Root { properties } = root_class {
            Ok(Self(properties))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid root class",
            ))
        }
    }
}
