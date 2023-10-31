//! File abstraction for reading from a PBO file.

use std::io::Read;

use crate::model::Header;

/// A file in a PBO
pub struct File<'a, I: Read> {
    size: u32,
    read: u32,
    input: &'a mut I,
}

impl<'a, I: Read> File<'a, I> {
    /// Create a new file from a header and a reader
    pub fn new(header: &Header, input: &'a mut I) -> Self {
        Self {
            size: header.size(),
            read: 0,
            input,
        }
    }
}

impl<I: Read> Read for File<'_, I> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // read up to the size of the file
        let size = std::cmp::min(self.size - self.read, buf.len() as u32);
        let read = self.input.read(&mut buf[..size as usize])?;
        self.read += read as u32;
        Ok(read)
    }
}
