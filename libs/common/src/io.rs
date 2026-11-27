//! IO utilities.

// Parts of the following code is derivative work of the code from the armake2 project by KoffeinFlummi,
// which is licensed GPLv2. This code therefore is also licensed under the terms
// of the GNU Public License, verison 2.

// The original code can be found here:
// https://github.com/KoffeinFlummi/armake2/blob/4b736afc8c615cf49a0d1adce8f6b9a8ae31d90f/src/io.rs

use std::io;
use std::io::{Read, Write};

/// Read extension trait
pub trait ReadExt: Read {
    /// Read a null-terminated string from the input.
    ///
    /// # Errors
    /// If the input fails to read, or the string is not valid UTF-8
    /// ([`io::ErrorKind::InvalidData`]).
    fn read_cstring(&mut self) -> io::Result<String>;
    /// Read a null-terminated string from the input, replacing invalid UTF-8.
    ///
    /// For names that are only displayed, where losing the value entirely is worse
    /// than showing a replacement character.
    ///
    /// # Errors
    /// If the input fails to read.
    fn read_cstring_lossy(&mut self) -> io::Result<String>;
    /// Read the bytes of a null-terminated string from the input, without the terminator.
    ///
    /// Reaching the end of the input also ends the string.
    ///
    /// # Errors
    /// If the input fails to read.
    fn read_cstring_bytes(&mut self) -> io::Result<Vec<u8>>;
    /// Reads a compressed `u32` from the input.
    ///
    /// # Errors
    /// If the input fails to read.
    fn read_compressed_int(&mut self) -> io::Result<u32>;
}

impl<T: Read> ReadExt for T {
    #[rustversion::attr(
        since(1.87),
        expect(
            clippy::unbuffered_bytes,
            reason = "The consumer should use a buffered reader if needed"
        )
    )]
    /// Reads a null-terminated string from the input.
    ///
    /// Uses `.bytes()` internally, a buffered reader should be used if performance is a concern.
    fn read_cstring_bytes(&mut self) -> io::Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();
        for byte in self.bytes() {
            let b = byte?;
            if b == 0 {
                break;
            }
            bytes.push(b);
        }
        Ok(bytes)
    }

    fn read_cstring(&mut self) -> io::Result<String> {
        String::from_utf8(self.read_cstring_bytes()?).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "string is not valid UTF-8: `{}`",
                    String::from_utf8_lossy(e.as_bytes())
                ),
            )
        })
    }

    fn read_cstring_lossy(&mut self) -> io::Result<String> {
        Ok(String::from_utf8_lossy(&self.read_cstring_bytes()?).into_owned())
    }

    #[rustversion::attr(
        since(1.87),
        expect(
            clippy::unbuffered_bytes,
            reason = "The consumer should use a buffered reader if needed"
        )
    )]
    /// Reads a null-terminated string from the input.
    ///
    /// Uses `.bytes()` internally, a buffered reader should be used if performance is a concern.
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

/// Write extension trait
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

    #[allow(clippy::cast_possible_truncation)]
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
/// Returns the length of a compressed `u32`.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cstring() {
        let mut input = std::io::Cursor::new(b"one\0two\0three".to_vec());
        assert_eq!(input.read_cstring().expect("read"), "one");
        assert_eq!(input.read_cstring().expect("read"), "two");
        assert_eq!(
            input.read_cstring().expect("read"),
            "three",
            "ends at end of input"
        );
        assert_eq!(input.read_cstring().expect("read"), "");
    }

    #[test]
    fn cstring_invalid_utf8() {
        let bytes = b"caf\xE9\0next\0".to_vec();
        let error = std::io::Cursor::new(bytes.clone())
            .read_cstring()
            .expect_err("invalid utf-8 is an error");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("caf\u{FFFD}"), "{error}");

        let mut input = std::io::Cursor::new(bytes);
        assert_eq!(input.read_cstring_lossy().expect("read"), "caf\u{FFFD}");
        assert_eq!(
            input.read_cstring_lossy().expect("read"),
            "next",
            "the terminator is consumed"
        );
    }

    #[test]
    fn compress_int() {
        for i in 0..0x000f_ffff {
            let expected_len = compressed_int_len(i);
            let mut buf = Vec::new();
            buf.write_compressed_int(i).expect("failed to write");
            assert_eq!(buf.len(), expected_len);
            let mut cursor = std::io::Cursor::new(buf);
            let result = cursor.read_compressed_int().expect("failed to read");
            assert_eq!(i, result);
        }
    }
}
