use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use byteorder::ReadBytesExt;
use hemtt_io::{ReadExt, WriteExt};
use indexmap::IndexMap;
use sha1::{Digest, Sha1};

use crate::{
    error::Error,
    file::File,
    model::{Checksum, Header, Mime},
    BISignVersion, ReadPbo, WritePbo,
};

pub struct ReadablePbo<I: Seek + Read> {
    extensions: IndexMap<String, String>,
    headers: Vec<Header>,
    checksum: Checksum,
    input: I,
    pub blob_start: u64,
}

impl<I: Seek + Read> ReadablePbo<I> {
    /// Read a PBO from a file
    ///
    /// # Errors
    /// if the file cannot be read
    pub fn from(mut input: I) -> Result<Self, Error> {
        let mut extensions = IndexMap::new();
        let mut headers = Vec::new();
        let mut blob_start = 0;
        loop {
            let (header, size) = Header::read_pbo(&mut input)?;
            blob_start += size as u64;
            if header.mime() == &Mime::Vers {
                loop {
                    let key = input.read_cstring()?;
                    blob_start += key.len() as u64 + 1;
                    if key.is_empty() {
                        break;
                    }
                    let value = input.read_cstring()?;
                    blob_start += value.len() as u64 + 1;
                    extensions.insert(key, value);
                }
            } else if header.filename().is_empty() {
                break;
            } else {
                headers.push(header);
            }
        }

        for header in &headers {
            input.seek(SeekFrom::Current(i64::from(header.size())))?;
        }

        input.seek(SeekFrom::Current(1))?;
        let checksum = Checksum::read_pbo(&mut input)?.0;
        if input.read_u8().is_ok() {
            return Err(Error::UnexpectedDataAfterChecksum);
        }
        Ok(Self {
            extensions,
            headers,
            checksum,
            input,
            blob_start,
        })
    }

    pub fn header(&self, name: &str) -> Option<&Header> {
        self.headers
            .iter()
            .find(|h| h.filename() == name.replace('/', "\\"))
    }

    pub const fn extensions(&self) -> &IndexMap<String, String> {
        &self.extensions
    }

    pub const fn checksum(&self) -> &Checksum {
        &self.checksum
    }

    pub fn files(&self) -> Vec<Header> {
        self.headers.clone()
    }

    pub fn files_sorted(&mut self) -> Vec<Header> {
        let mut sorted = self.files();
        sorted.sort_by(|a, b| {
            a.filename()
                .to_lowercase()
                .cmp(&b.filename().to_lowercase())
        });
        sorted
    }

    /// Read a file from the PBO
    ///
    /// # Errors
    /// if the file cannot be read
    pub fn file(&mut self, name: &str) -> Result<Option<File<I>>, Error> {
        self.input.seek(SeekFrom::Start(self.blob_start))?;
        for header in &self.headers {
            if header.filename().to_lowercase() == name.replace('/', "\\").to_lowercase() {
                return Ok(Some(File::new(header, &mut self.input)));
            }
            self.input
                .seek(SeekFrom::Current(i64::from(header.size())))?;
        }
        Ok(None)
    }

    /// Check if the files are sorted correctly
    ///
    /// # Errors
    /// if the files are not sorted correctly
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

    /// Generate a checksum for the PBO
    ///
    /// # Errors
    /// if the pbo cannot be read
    ///
    /// # Panics
    /// if a file does not exist, but a header for it does
    pub fn gen_checksum(&mut self) -> Result<Checksum, Error> {
        let mut headers: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        Header::ext().write_pbo(&mut headers)?;

        if let Some(prefix) = self.extensions.get("prefix") {
            headers.write_cstring(b"prefix")?;
            headers.write_cstring(prefix)?;
        }

        for (key, value) in &self.extensions {
            if key == "prefix" {
                continue;
            }
            headers.write_cstring(key.as_bytes())?;
            headers.write_cstring(value.as_bytes())?;
        }

        headers.write_all(&[0])?;

        for header in &self.files_sorted() {
            header.write_pbo(&mut headers)?;
        }

        Header::default().write_pbo(&mut headers)?;

        let mut hasher = Sha1::new();

        hasher.update(headers.get_ref());

        for header in &self.files_sorted() {
            let mut file = self.file(header.filename())?.unwrap();
            std::io::copy(&mut file, &mut hasher)?;
        }

        Ok(hasher.finalize().to_vec().into())
    }

    /// Hashes all the files names in a PBO, expects the PBO to be sorted
    /// Empty files are not hashed
    ///
    /// # Errors
    /// if the pbo cannot be read
    pub fn hash_filenames(&mut self) -> Result<Checksum, Error> {
        let mut hasher = Sha1::new();

        for header in &self.files_sorted() {
            // Skip empty files
            let Some(mut file) = self.file(header.filename())? else {
                continue;
            };
            if file.read_u8().is_err() {
                continue;
            }
            hasher.update(header.filename().replace('/', "\\").to_lowercase());
        }

        Ok(hasher.finalize().to_vec().into())
    }

    /// Hashes all the files in a PBO, expects the PBO to be sorted
    ///
    /// # Errors
    /// if the pbo cannot be read
    pub fn hash_files(&mut self, version: BISignVersion) -> Result<Checksum, Error> {
        let mut hasher = Sha1::new();

        if self.files().is_empty() {
            hasher.update(version.nothing());
        }

        for header in &self.files_sorted() {
            if !version.should_hash_file(header.filename()) {
                continue;
            }
            let Some(mut file) = self.file(header.filename())? else {
                continue;
            };
            std::io::copy(&mut file, &mut hasher)?;
        }

        Ok(hasher.finalize().to_vec().into())
    }
}
