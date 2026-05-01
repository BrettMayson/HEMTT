//! File abstraction for reading from a PBO file.

use std::io::Read;

use crate::model::Header;

/// A file in a PBO
pub struct File<'a, I: Read> {
    size: u32,
    read: u32,
    decompressed: Option<Vec<u8>>,
    input: &'a mut I,
}

impl<'a, I: Read> File<'a, I> {
    /// Create a new file from a header and a reader
    ///
    /// # Panics
    /// If the file is compressed and the compressed data cannot be read or decompressed
    pub fn new(header: &Header, input: &'a mut I) -> Self {
        Self {
            size: if header.mime().is_compressed() {
                header.original()
            } else {
                header.size()
            },
            read: 0,
            decompressed: if header.mime().is_compressed() {
                let mut compressed = vec![0; header.size() as usize];
                input
                    .read_exact(&mut compressed)
                    .expect("Failed to read compressed file");
                let mut decompressed = vec![0; header.original() as usize];
                hemtt_lzo::lz77::decompress(&compressed, &mut decompressed)
                    .expect("Failed to decompress file");
                Some(decompressed)
            } else {
                None
            },
            input,
        }
    }
}

impl<I: Read> Read for File<'_, I> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // read up to the size of the file
        let size = std::cmp::min(self.size - self.read, buf.len() as u32);
        let read = if let Some(decompressed) = &mut self.decompressed {
            let size = std::cmp::min(size as usize, decompressed.len());
            buf[..size].copy_from_slice(&decompressed[..size]);
            decompressed.drain(..size);
            size
        } else {
            self.input.read(&mut buf[..size as usize])?
        };
        self.read += read as u32;
        Ok(read)
    }
}
