use std::io::{Cursor, Error, Read, Seek, SeekFrom, Write};

use hemtt_io::*;
use indexmap::IndexMap;
use sha1::{Digest, Sha1};

use crate::{Header, Timestamp};

#[derive(Default)]
pub struct ReadablePbo<I: Seek + Read> {
    extensions: IndexMap<String, String>,
    headers: Vec<Header>,
    checksum: Vec<u8>,
    input: I,
    blob_start: u64,
}

impl<I: Seek + Read> ReadablePbo<I> {
    /// Open a PBO
    pub fn from(input: I) -> Result<Self, Error> {
        let mut pbo = Self {
            extensions: IndexMap::new(),
            headers: Vec::new(),
            checksum: Vec::new(),
            input,
            blob_start: 0,
        };
        loop {
            let (header, size) = Header::read(&mut pbo.input)?;
            pbo.blob_start += size as u64;
            if header.method() == 0x5665_7273 {
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
            } else if header.filename().is_empty() {
                break;
            } else {
                pbo.headers.push(header);
            }
        }

        for header in &pbo.headers {
            pbo.input
                .seek(SeekFrom::Current(i64::from(header.size())))
                .unwrap();
        }

        pbo.input.seek(SeekFrom::Current(1))?;
        let mut checksum = vec![0; 20];
        pbo.input.read_exact(&mut checksum)?;
        pbo.checksum = checksum;
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
    pub fn is_sorted(&self) -> Result<(), (Vec<Header>, Vec<Header>)> {
        fn compare(a: &Header, b: &Header) -> std::cmp::Ordering {
            a.filename()
                .to_lowercase()
                .cmp(&b.filename().to_lowercase())
        }
        let files = self.files();
        let mut files = files.iter();
        let mut last = match files.next() {
            Some(e) => e,
            None => return Ok(()),
        };

        for curr in files {
            if compare(last, curr) == std::cmp::Ordering::Greater {
                return Err((self.files(), {
                    let mut files = self.files();
                    files.sort_by(compare);
                    files
                }));
            }
            last = curr;
        }
        Ok(())
    }

    pub fn extensions(&self) -> &IndexMap<String, String> {
        &self.extensions
    }

    /// Finds a header if it exists
    pub fn header(&mut self, filename: &str) -> Option<Header> {
        for header in &self.headers {
            if header.filename() == filename.replace("/", "\\").as_str() {
                return Some(header.clone());
            }
        }
        None
    }

    pub fn extension(&self, key: &str) -> Option<&String> {
        self.extensions.get(key)
    }

    pub fn checksum(&self) -> Vec<u8> {
        self.checksum.clone()
    }

    /// Generate a checksum of the PBO
    pub fn gen_checksum(&mut self) -> std::io::Result<Vec<u8>> {
        let mut headers: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let ext_header = Header {
            filename: String::new(),
            method: 0x5665_7273,
            original: 0,
            reserved: 0,
            timestamp: Timestamp::from_u32(0),
            size: 0,
        };
        ext_header.write(&mut headers)?;

        if let Some(prefix) = self.extensions.get("prefix") {
            headers.write_all(b"prefix\0")?;
            headers.write_cstring(prefix)?;
        }

        for (key, value) in &self.extensions {
            if key == "prefix" {
                continue;
            }
            headers.write_cstring(key)?;
            headers.write_cstring(value)?;
        }
        headers.write_all(b"\0")?;

        let files = self.files();

        for header in &files {
            header.write(&mut headers)?;
        }

        let header = Header {
            method: 0,
            ..ext_header
        };
        header.write(&mut headers)?;

        let mut h = Sha1::new();

        h.update(headers.get_ref());

        for header in &files {
            let cursor = self.retrieve(header.filename()).unwrap();
            h.update(cursor.get_ref());
        }

        Ok(h.finalize().to_vec())
    }

    /// Retrieves a file from a PBO
    pub fn retrieve(&mut self, filename: &str) -> Option<Cursor<Vec<u8>>> {
        let filename_owned = filename.replace("/", "\\");
        let filename = filename_owned.as_str();
        self.input.seek(SeekFrom::Start(self.blob_start)).unwrap();
        for h in &self.headers {
            if h.filename().to_lowercase() == filename.to_lowercase() {
                let mut buffer: Vec<u8> = Vec::new();
                self.input.read_exact(&mut buffer).unwrap();
                return Some(Cursor::new(buffer));
            } else {
                self.input
                    .seek(SeekFrom::Current(i64::from(h.size())))
                    .unwrap();
            }
        }
        None
    }
}
