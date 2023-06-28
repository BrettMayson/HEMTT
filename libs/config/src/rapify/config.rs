use std::io::Cursor;

use byteorder::{LittleEndian, WriteBytesExt};

use crate::{Class, Config};

use super::Rapify;

impl Rapify for Config {
    fn rapify<O: std::io::Write>(
        &self,
        output: &mut O,
        _offset: usize,
    ) -> Result<usize, std::io::Error> {
        output.write_all(b"\0raP")?;
        output.write_all(b"\0\0\0\0\x08\0\0\0")?;

        let root_class = Class::Local {
            name: crate::Ident::default(),
            parent: None,
            properties: self.0.clone(),
        };
        let buffer: Box<[u8]> = vec![0; root_class.rapified_length()].into_boxed_slice();
        let mut cursor = Cursor::new(buffer);
        let written = root_class.rapify(&mut cursor, 16)?;
        // assert_eq!(written, self.root.rapified_length());

        let enum_offset = 16 + cursor.get_ref().len() as u32;
        output.write_u32::<LittleEndian>(enum_offset)?;

        output.write_all(cursor.get_ref())?;

        output.write_all(b"\0\0\0\0")?;
        Ok(written + 4)
    }

    fn rapified_length(&self) -> usize {
        let root_class = Class::Local {
            name: crate::Ident::default(),
            parent: None,
            properties: self.0.clone(),
        };
        20 + root_class.rapified_length()
    }
}
