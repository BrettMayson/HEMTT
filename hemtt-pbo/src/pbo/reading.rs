use std::io::{Cursor, Error, Read, Seek, SeekFrom};

use hemtt_io::*;
use indexmap::IndexMap;

use crate::Header;

use super::WritablePBO;

#[derive(Default)]
pub struct ReadablePBO<I: Seek + Read> {
    extensions: IndexMap<String, String>,
    headers: Vec<Header>,
    checksum: Option<Vec<u8>>,
    input: I,
    blob_start: u64,
}

impl<I: Seek + Read> ReadablePBO<I> {
    /// Open a PBO
    pub fn from(input: I) -> Result<Self, Error> {
        let mut pbo = Self {
            extensions: IndexMap::new(),
            headers: Vec::new(),
            checksum: None,
            input,
            blob_start: 0,
        };
        loop {
            let (header, size) = Header::read(&mut pbo.input)?;
            pbo.blob_start += size as u64;
            if header.method == 0x5665_7273 {
                loop {
                    let s = pbo.input.read_cstring()?;
                    pbo.blob_start += s.as_bytes().len() as u64 + 1;
                    if s.is_empty() {
                        break;
                    }
                    let val = pbo.input.read_cstring()?;
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
            pbo.input
                .seek(SeekFrom::Current(i64::from(header.size)))
                .unwrap();
        }

        pbo.input.seek(SeekFrom::Current(1))?;
        let mut checksum = vec![0; 20];
        pbo.input.read_exact(&mut checksum)?;
        pbo.checksum = Some(checksum);
        if let Ok(u) = pbo.input.read(&mut Vec::with_capacity(1)) {
            if u > 0 {
                error!("Unexpected data after reading checksum");
            }
        }
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

    /// Returns if the files are sorted into the correct order
    pub fn is_sorted(&self) -> bool {
        // self.files().is_sorted_by(|a, b| a.filename.to_lowercase().cmp(&b.filename.to_lowercase()))
        fn compare(a: &&Header, b: &&Header) -> Option<std::cmp::Ordering> {
            Some(a.filename.to_lowercase().cmp(&b.filename.to_lowercase()))
        }
        let sorted = self.files();
        let mut sorted = sorted.iter();
        let mut last = match sorted.next() {
            Some(e) => e,
            None => return true,
        };

        for curr in sorted {
            if let Some(std::cmp::Ordering::Greater) | None = compare(&last, &curr) {
                return false;
            }
            last = curr;
        }

        true
    }

    pub fn extensions(&self) -> &IndexMap<String, String> {
        &self.extensions
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

    pub fn extension(&self, key: &str) -> Option<&String> {
        self.extensions.get(key)
    }

    pub fn checksum(&self) -> Option<Vec<u8>> {
        self.checksum.clone()
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

impl<B: Seek + Read> Into<WritablePBO<Cursor<Box<[u8]>>>> for ReadablePBO<B> {
    fn into(mut self) -> WritablePBO<Cursor<Box<[u8]>>> {
        let mut pbo = WritablePBO::new();
        for header in self.files() {
            pbo.add_file(&header.filename, self.retrieve(&header.filename).unwrap(), header.clone())
                .unwrap();
        }
        for (key, value) in self.extensions {
            pbo.add_extension(key, value);
        }
        pbo
    }
}
