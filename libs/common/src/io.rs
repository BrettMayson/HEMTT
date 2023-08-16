#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

// Parts of the following code is derivative work of the code from the armake2 project by KoffeinFlummi,
// which is licensed GPLv2. This code therefore is also licensed under the terms
// of the GNU Public License, verison 2.

// The original code can be found here:
// https://github.com/KoffeinFlummi/armake2/blob/4b736afc8c615cf49a0d1adce8f6b9a8ae31d90f/src/io.rs

use std::io;
use std::io::{Read, Write};

pub trait ReadExt: Read {
    /// Read a null-terminated string from the input.
    ///
    /// # Errors
    /// If the input fails to read.
    fn read_cstring(&mut self) -> io::Result<String>;
    /// Reads a compressed `u32` from the input.
    ///
    /// # Errors
    /// If the input fails to read.
    fn read_compressed_int(&mut self) -> io::Result<u32>;
}

impl<T: Read> ReadExt for T {
    fn read_cstring(&mut self) -> io::Result<String> {
        let mut bytes: Vec<u8> = Vec::new();
        for byte in self.bytes() {
            let b = byte?;
            if b == 0 {
                break;
            }
            bytes.push(b);
        }

        Ok(String::from_utf8(bytes).unwrap())
    }

    fn read_compressed_int(&mut self) -> io::Result<u32> {
        let mut result: u32 = 0;
        for (i, byte) in self.bytes().enumerate() {
            let b: u32 = byte?.into();
            result |= (b & 0x7f) << (i * 7);
            if b < 0x80 {
                break;
            }
        }
        Ok(result)
    }
}

pub trait WriteExt: Write {
    /// Writes a null-terminated string to the output.
    ///
    /// # Errors
    /// If the output fails to write.
    fn write_cstring<S: AsRef<[u8]>>(&mut self, s: S) -> io::Result<()>;
    /// Writes a compressed `u32` to the output.
    ///
    /// # Errors
    /// If the output fails to write.
    fn write_compressed_int(&mut self, x: u32) -> io::Result<usize>;
}

impl<T: Write> WriteExt for T {
    fn write_cstring<S: AsRef<[u8]>>(&mut self, s: S) -> io::Result<()> {
        self.write_all(s.as_ref())?;
        self.write_all(b"\0")?;
        Ok(())
    }

    fn write_compressed_int(&mut self, x: u32) -> io::Result<usize> {
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
