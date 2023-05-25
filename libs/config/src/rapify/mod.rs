//! Rapify configs to binary

use std::io::{Cursor, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::Config;

mod array;
mod class;
mod entry;
mod number;
mod str;

/// Trait for rapifying objects
pub trait Rapify {
    /// Rapify the object into the output stream
    ///
    /// # Errors
    /// if the output stream fails
    fn rapify<O: Write>(&self, output: &mut O, offset: usize) -> Result<usize, std::io::Error>;
    /// Get the length of the rapified object
    fn rapified_length(&self) -> usize;
    /// Get the rapified element code
    fn rapified_code(&self) -> u8 {
        3
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

trait WriteExt: Write {
    fn write_cstring<S: AsRef<[u8]>>(&mut self, s: S) -> std::io::Result<()>;
    fn write_compressed_int(&mut self, x: u32) -> std::io::Result<usize>;
}

impl<T: Write> WriteExt for T {
    fn write_cstring<S: AsRef<[u8]>>(&mut self, s: S) -> std::io::Result<()> {
        self.write_all(s.as_ref())?;
        self.write_all(b"\0")?;
        Ok(())
    }

    fn write_compressed_int(&mut self, x: u32) -> std::io::Result<usize> {
        let mut temp = x;
        let mut len = 0;

        while temp > 0x7f {
            self.write_all(&[(0x80 | temp & 0x7f) as u8])?;
            len += 1;
            temp &= !0x7f;
            temp >>= 7;
        }

        self.write_all(&[temp as u8])?;
        Ok(len + 1)
    }
}

#[must_use]
/// Get the length of a compressed integer
pub const fn compressed_int_len(x: u32) -> usize {
    let mut temp = x;
    let mut len = 0;

    while temp > 0x7f {
        len += 1;
        temp &= !0x7f;
        temp >>= 7;
    }

    len + 1
}
