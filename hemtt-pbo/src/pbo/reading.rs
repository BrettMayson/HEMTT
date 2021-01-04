use std::io::{Cursor, Error, Read, Seek, SeekFrom};

use hemtt_io::*;
use indexmap::IndexMap;

use crate::Header;

use super::WritablePBO;

#[derive(Default)]
pub struct ReadablePBO<I: Seek + Read + Copy> {
    extensions: IndexMap<String, String>,
    headers: Vec<Header>,
    checksum: Option<Vec<u8>>,
    input: I,
    blob_start: u64,
}

impl<I: Seek + Read + Copy> ReadablePBO<I> {
    /// Create a pbo object by reading a file
    pub fn from(mut input: I) -> Result<Self, Error> {
        let mut pbo = Self {
            extensions: IndexMap::new(),
            headers: Vec::new(),
            checksum: None,

            input,
            blob_start: 0,
        };
        loop {
            let (header, size) = Header::read(&mut input)?;
            pbo.blob_start += size as u64;
            if header.method == 0x5665_7273 {
                loop {
                    let s = input.read_cstring()?;
                    pbo.blob_start += s.as_bytes().len() as u64 + 1;
                    if s.is_empty() {
                        break;
                    }
                    let val = input.read_cstring()?;
                    pbo.blob_start += val.as_bytes().len() as u64 + 1;
                    pbo.extensions.insert(s.clone(), val);
                }
            } else if header.filename.is_empty() {
                break;
            } else {
                pbo.headers.push(header);
            }
        }

        for header in &pbo.headers {
            input
                .seek(SeekFrom::Current(i64::from(header.size)))
                .unwrap();
        }

        input.seek(SeekFrom::Current(1))?;
        let mut checksum = vec![0; 20];
        input.read_exact(&mut checksum)?;
        pbo.checksum = Some(checksum);

        Ok(pbo)
    }

    /// A list of filenames in the PBO
    pub fn files(&self) -> Vec<Header> {
        let mut filenames = Vec::new();
        for h in &self.headers {
            filenames.push(h.clone());
        }
        filenames
    }

    /// Get files in alphabetical order
    pub fn files_sorted(&self) -> Vec<Header> {
        let mut sorted = self.files();
        sorted.sort_by(|a, b| a.filename.to_lowercase().cmp(&b.filename.to_lowercase()));
        sorted
    }

    /// Finds a header if it exists
    pub fn header(&mut self, filename: &str) -> Option<Header> {
        for header in &self.headers {
            if header.filename == filename.replace("/", "\\").as_str() {
                return Some(header.clone());
            }
        }
        None
    }

    /// Retrieves a file from a PBO
    pub fn retrieve(&mut self, filename: &str) -> Option<Cursor<Box<[u8]>>> {
        let filename_owned = filename.replace("/", "\\");
        let filename = filename_owned.as_str();
        self.input.seek(SeekFrom::Start(self.blob_start)).unwrap();
        for h in &self.headers {
            if h.filename == filename {
                let mut buffer: Box<[u8]> = vec![0; h.size as usize].into_boxed_slice();
                self.input.read_exact(&mut buffer).unwrap();
                return Some(Cursor::new(buffer));
            } else {
                self.input
                    .seek(SeekFrom::Current(i64::from(h.size)))
                    .unwrap();
            }
        }
        None
    }
}

impl<B: Seek + Read + Copy> Into<WritablePBO<Cursor<Box<[u8]>>>> for ReadablePBO<B> {
    fn into(mut self) -> WritablePBO<Cursor<Box<[u8]>>> {
        let mut pbo = WritablePBO::new();
        for header in self.files_sorted() {
            pbo.add_file(&header.filename, self.retrieve(&header.filename).unwrap()).unwrap();
        }
        for (key, value) in self.extensions {
            pbo.add_extension(key, value);
        }
        pbo
    }
}
