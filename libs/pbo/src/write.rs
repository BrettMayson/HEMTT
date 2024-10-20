use std::{
    collections::HashMap,
    io::{BufWriter, Cursor, Read, Seek, SeekFrom, Write},
};

use hemtt_common::io::WriteExt;
use indexmap::IndexMap;
use sha1::{Digest, Sha1};

use crate::{error::Error, model::Header, Checksum, WritePbo};

#[derive(Default)]
/// A PBO file that can be written to
pub struct WritablePbo<I: Seek + Read> {
    properties: IndexMap<String, String>,
    files: HashMap<String, (I, Header)>,
}

impl<I: Seek + Read> WritablePbo<I> {
    #[must_use]
    /// Create a new PBO
    pub fn new() -> Self {
        Self {
            properties: IndexMap::new(),
            files: HashMap::new(),
        }
    }

    /// Add files to the PBO
    ///
    /// # Errors
    /// if the file cannot be read
    pub fn add_file<S: Into<String>>(
        &mut self,
        name: S,
        mut input: I,
    ) -> Result<Option<(I, Header)>, Error> {
        let name = name.into().replace('/', "\\");
        let size = input.seek(SeekFrom::End(0))?;
        if size > u32::MAX as u64 {
            return Err(Error::FileTooLarge);
        }
        Ok(self.files.insert(
            name.clone(),
            (input, Header::new_for_file(name, size as u32)),
        ))
    }

    /// Add a file with a custom header
    ///
    /// # Errors
    /// if the file cannot be read
    pub fn add_file_with_header(
        &mut self,
        header: Header,
        input: I,
    ) -> Result<Option<(I, Header)>, Error> {
        let name = header.filename().replace('/', "\\");
        Ok(self.files.insert(name, (input, header)))
    }

    /// Read a file from the PBO
    ///
    /// # Errors
    /// if the file cannot be read
    pub fn file(&mut self, name: &str) -> Result<Option<&mut I>, Error> {
        let name = name.replace('/', "\\");
        if let Some((input, _)) = self.files.get_mut(&name) {
            input.rewind()?;
            Ok(Some(input))
        } else {
            Ok(None)
        }
    }

    /// Get a list of all files in the PBO
    #[must_use]
    pub fn files(&self) -> Vec<Header> {
        let mut filenames = Vec::new();
        for (_, h) in self.files.values() {
            filenames.push(h.clone());
        }
        filenames
    }

    /// Get a list of all files in the PBO sorted by name
    #[must_use]
    pub fn files_sorted(&self) -> Vec<Header> {
        let mut sorted = self.files();
        sorted.sort_by(|a, b| {
            a.filename()
                .to_lowercase()
                .cmp(&b.filename().to_lowercase())
        });
        sorted
    }

    /// Add an property to the PBO
    pub fn add_property<K: Into<String>, V: Into<String>>(
        &mut self,
        key: K,
        value: V,
    ) -> Option<String> {
        self.properties
            .insert(key.into(), value.into().trim_matches('\\').to_string())
    }

    /// Remove an property from the PBO
    pub fn remove_property(&mut self, key: &str) -> Option<String> {
        self.properties.swap_remove(key)
    }

    #[must_use]
    /// Get an property from the PBO
    pub fn property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }

    #[must_use]
    /// Get all properties from the PBO
    pub const fn properties(&self) -> &IndexMap<String, String> {
        &self.properties
    }

    /// Write the PBO to a file
    ///
    /// # Errors
    /// if the file cannot be written
    ///
    /// # Panics
    /// if a file does not exist but a header is present
    pub fn write<O: Write + Send>(
        &mut self,
        output: &mut O,
        properties: bool,
    ) -> Result<Checksum, Error> {
        let mut headers: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        if properties {
            Header::property().write_pbo(&mut headers)?;

            if let Some(prefix) = self.property("prefix") {
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
        }

        let files_sorted = self.files_sorted();

        for header in &files_sorted {
            header.write_pbo(&mut headers)?;
        }

        Header::default().write_pbo(&mut headers)?;

        let mut hasher = Sha1::new();

        let mut buffered_output = BufWriter::new(output);

        buffered_output.write_all(headers.get_ref())?;
        hasher.update(headers.get_ref());

        for header in &files_sorted {
            let file = self
                .file(header.filename())?
                .expect("file with header should exist");
            let mut buffer = Vec::with_capacity(header.size() as usize);
            file.read_to_end(&mut buffer)?;

            if header.size() > 1_000_000 {
                // pay the paralellization cost for large files
                std::thread::scope(|s| {
                    s.spawn(|| {
                        hasher.update(&*buffer);
                    });
                    s.spawn(|| {
                        buffered_output
                            .write_all(&buffer)
                            .expect("failed to write file");
                    });
                });
            } else {
                buffered_output.write_all(&buffer)?;
                hasher.update(&buffer);
            }
        }

        let checksum = hasher.finalize();
        buffered_output.write_all(&[0])?;
        buffered_output.write_all(&checksum)?;
        buffered_output.flush()?;

        Ok(checksum.to_vec().into())
    }
}
