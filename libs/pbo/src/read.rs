use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use byteorder::ReadBytesExt;
use hemtt_common::io::{ReadExt, WriteExt};
use indexmap::IndexMap;
use sha1::{Digest, Sha1};

use crate::{
    error::Error,
    file::File,
    model::{Checksum, Header, Mime},
    BISignVersion, ReadPbo, WritePbo,
};

/// An existing PBO file that can be read from
pub struct ReadablePbo<I: Seek + Read> {
    properties: IndexMap<String, String>,
    headers: Vec<Header>,
    checksum: Checksum,
    input: I,
    blob_start: u64,
}

impl<I: Seek + Read> ReadablePbo<I> {
    /// Read a PBO from a file
    ///
    /// # Errors
    /// if the file cannot be read
    pub fn from(mut input: I) -> Result<Self, Error> {
        let mut properties = IndexMap::new();
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
                    properties.insert(key, value);
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
            properties,
            headers,
            checksum,
            input,
            blob_start,
        })
    }

    /// Find a header by name
    pub fn header(&self, name: &str) -> Option<&Header> {
        self.headers
            .iter()
            .find(|h| h.filename() == name.replace('/', "\\"))
    }

    /// Get the PBO's properties
    pub const fn properties(&self) -> &IndexMap<String, String> {
        &self.properties
    }

    /// Get the PBO's stored checksum
    pub const fn checksum(&self) -> &Checksum {
        &self.checksum
    }

    /// Get the PBO's headers
    pub fn files(&self) -> Vec<Header> {
        self.headers.clone()
    }

    /// Get the PBO's headers sorted by name
    pub fn files_sorted(&self) -> Vec<Header> {
        let mut sorted = self.files();
        sorted.sort_by(|a, b| {
            a.filename()
                .to_lowercase()
                .cmp(&b.filename().to_lowercase())
        });
        tracing::trace!("Sorted {} files", sorted.len());
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

    /// Find the offset of a file
    ///
    /// # Errors
    /// if the file cannot be read
    pub fn file_offset(&self, name: &str) -> Result<Option<u64>, Error> {
        let mut offset = self.blob_start;
        for header in &self.headers {
            if header.filename().to_lowercase() == name.replace('/', "\\").to_lowercase() {
                return Ok(Some(offset));
            }
            offset += header.size() as u64;
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
        let Some(mut last) = files.next() else {
            return Ok(());
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
        Header::property().write_pbo(&mut headers)?;

        if let Some(prefix) = self.properties.get("prefix") {
            headers.write_cstring(b"prefix")?;
            headers.write_cstring(prefix)?;
        }

        for (key, value) in &self.properties {
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
            let mut file = self
                .file(header.filename())?
                .expect("file with header should exist");
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

        if self.files().is_empty() {
            return Err(Error::NoFiles);
        }

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

        let mut nothing = true;

        for header in &self.files_sorted() {
            if !version.should_hash_file(header.filename()) {
                continue;
            }
            nothing = false;
            let Some(mut file) = self.file(header.filename())? else {
                continue;
            };
            std::io::copy(&mut file, &mut hasher)?;
        }

        if nothing {
            hasher.update(version.nothing());
        }

        Ok(hasher.finalize().to_vec().into())
    }
}
