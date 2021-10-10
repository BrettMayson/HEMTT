use std::io::{Cursor, Read, Result, Seek};
use std::{
    collections::HashMap,
    io::{SeekFrom, Write},
};

use hemtt_io::WriteExt;
use indexmap::IndexMap;
use sha1::{Digest, Sha1};

use crate::{Header, ReadablePbo, Timestamp};

#[derive(Default)]
pub struct WritablePbo<I: Seek + Read> {
    extensions: IndexMap<String, String>,
    files: HashMap<String, (I, Header)>,
}

impl<I: Seek + Read> WritablePbo<I> {
    /// Create an empty PBO for writing
    pub fn new() -> Self {
        Self {
            extensions: IndexMap::new(),
            files: HashMap::new(),
        }
    }

    /// A list of filenames in the PBO
    pub fn files(&mut self) -> Result<Vec<Header>> {
        let mut filenames = Vec::new();
        for (_, h) in self.files.values() {
            // let size = c.seek(SeekFrom::End(0))? as u32;
            filenames.push(h.clone());
        }
        Ok(filenames)
    }

    /// Get files in alphabetical order
    pub fn files_sorted(&mut self) -> Result<Vec<Header>> {
        let mut sorted = self.files()?;
        sorted.sort_by(|a, b| {
            a.filename()
                .to_lowercase()
                .cmp(&b.filename().to_lowercase())
        });
        Ok(sorted)
    }

    /// Removes a file, returning it if it existed
    pub fn remove_file<S: Into<String>>(&mut self, filename: S) -> Option<(I, Header)> {
        let filename = filename.into();
        trace!("removing file from struct: {}", filename);
        self.files.remove(&filename.replace("/", "\\"))
    }

    /// Adds or updates a file to the PBO, returns the old file if it existed
    pub fn add_file<S: Into<String>>(
        &mut self,
        filename: S,
        mut file: I,
    ) -> Result<Option<(I, Header)>> {
        let filename = filename.into().replace("/", "\\");
        trace!("adding file to struct: {}", filename);
        let size = file.seek(SeekFrom::End(0))? as u32;
        Ok(self.files.insert(
            filename.replace("/", "\\"),
            (
                file,
                Header {
                    filename,
                    method: 0,
                    original: size,
                    reserved: 0,
                    timestamp: Timestamp::from_u32(0),
                    size,
                },
            ),
        ))
    }

    /// Adds or updates a file with an existing header to the PBO, returns the old file if it existed
    pub fn add_file_header<S: Into<String>>(
        &mut self,
        filename: S,
        mut file: I,
        header: Header,
    ) -> Result<Option<(I, Header)>> {
        let filename = filename.into();
        trace!("adding file to struct: {}", filename);
        let size = file.seek(SeekFrom::End(0))?;
        if size > u32::MAX as u64 {
            Err(std::io::Error::from(std::io::ErrorKind::Other))
        } else {
            Ok(self
                .files
                .insert(filename.replace("/", "\\"), (file, header)))
        }
    }

    /// Retrieves a file from a PBO
    pub fn retrieve_file<S: Into<String>>(
        &mut self,
        filename: S,
    ) -> Result<Option<Cursor<Box<[u8]>>>> {
        let filename_owned = filename.into().replace("/", "\\");
        let filename = filename_owned.as_str();
        if self.files.contains_key(filename) {
            let (mut data, header) = self.files.remove(filename).unwrap();
            let mut buffer: Box<[u8]> =
                vec![0; data.seek(SeekFrom::End(0))? as usize].into_boxed_slice();
            data.seek(SeekFrom::Start(0))?;
            data.read_exact(&mut buffer)?;
            self.files.insert(filename.to_string(), (data, header));
            return Ok(Some(Cursor::new(buffer)));
        }
        Ok(None)
    }

    /// Add an extension to the PBO
    pub fn add_extension<K: Into<String>, V: Into<String>>(
        &mut self,
        key: K,
        value: V,
    ) -> Option<String> {
        self.extensions
            .insert(key.into(), value.into().trim_matches('\\').to_string())
    }

    pub fn remove_extension(&mut self, key: &str) -> Option<String> {
        self.extensions.remove(key)
    }

    pub fn extensions(&self) -> &IndexMap<String, String> {
        &self.extensions
    }

    /// Write the PBO file
    pub fn write<O: Write>(&mut self, output: &mut O) -> Result<()> {
        let mut headers: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let ext_header = Header {
            filename: String::new(),
            method: 0x5665_7273,
            original: 0,
            reserved: 0,
            timestamp: Timestamp::from_u32(0),
            size: 0,
        };
        trace!("writing ext header: {:?}", ext_header);
        ext_header.write(&mut headers)?;

        if let Some(prefix) = self.extensions.get("prefix") {
            trace!("writing prefix header: {:?}", prefix);
            headers.write_all(b"prefix\0")?;
            headers.write_cstring(prefix)?;
        } else {
            trace!("no prefix header")
        }

        for (key, value) in &self.extensions {
            if key == "prefix" {
                continue;
            }
            trace!("writing `{}` header: {:?}", key, value);
            headers.write_cstring(key)?;
            headers.write_cstring(value)?;
        }
        headers.write_all(b"\0")?;

        let files_sorted = self.files_sorted()?;

        for header in &files_sorted {
            header.write(&mut headers)?;
        }

        let header = Header {
            method: 0,
            ..ext_header
        };
        trace!("writing null header");
        header.write(&mut headers)?;

        let mut h = Sha1::new();

        output.write_all(headers.get_ref())?;
        h.update(headers.get_ref());

        for header in &files_sorted {
            trace!("writing file {}", header.filename());
            let cursor = self.retrieve_file(header.filename())?.unwrap();
            output.write_all(cursor.get_ref())?;
            trace!("hashing");
            h.update(cursor.get_ref());
        }

        output.write_all(&[0])?;
        let hash = h.finalize().to_vec();
        debug!("pbo generated hash: {:?}", hash);
        output.write_all(&hash)?;

        Ok(())
    }

    /// Generate a checksum of the PBO
    pub fn checksum(&mut self) -> Result<Vec<u8>> {
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

        let files_sorted = self.files_sorted()?;

        for header in &files_sorted {
            header.write(&mut headers)?;
        }

        let header = Header {
            method: 0,
            ..ext_header
        };
        header.write(&mut headers)?;

        let mut h = Sha1::new();

        h.update(headers.get_ref());

        for header in &files_sorted {
            let cursor = self.retrieve_file(header.filename())?.unwrap();
            h.update(cursor.get_ref());
        }

        Ok(h.finalize().to_vec())
    }
}

impl<B: Seek + Read> From<ReadablePbo<B>> for WritablePbo<Cursor<Vec<u8>>> {
    fn from(mut rp: ReadablePbo<B>) -> Self {
        let mut pbo = Self::new();
        for header in rp.files() {
            pbo.add_file_header(
                header.filename(),
                rp.retrieve(header.filename()).unwrap(),
                header.clone(),
            )
            .unwrap();
        }
        for (key, value) in rp.extensions() {
            pbo.add_extension(key, value);
        }
        pbo
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::WritablePbo;

    #[test]
    fn empty_pbo() {
        let mut pbo = WritablePbo::<Cursor<Vec<u8>>>::new();
        let mut buffer = Vec::new();
        pbo.write(&mut Cursor::new(&mut buffer)).unwrap();
        assert_eq!(
            pbo.checksum().unwrap(),
            vec![
                68, 142, 162, 133, 179, 224, 152, 229, 10, 109, 120, 136, 145, 22, 232, 206, 165,
                206, 130, 23
            ]
        );
    }

    #[test]
    fn extensions() {
        let mut pbo = WritablePbo::<Cursor<Vec<u8>>>::new();
        pbo.add_extension("prefix", "foobar");
        pbo.add_extension("version", "1.2.3");
        let mut buffer = Vec::new();
        pbo.write(&mut Cursor::new(&mut buffer)).unwrap();
        assert_eq!(
            pbo.checksum().unwrap(),
            vec![
                82, 23, 199, 92, 103, 114, 230, 42, 118, 31, 171, 248, 220, 143, 202, 47, 233, 33,
                217, 248
            ]
        );
    }

    #[test]
    fn files() {
        let mut pbo = WritablePbo::<Cursor<Vec<u8>>>::new();
        pbo.add_extension("prefix", "foobar");
        pbo.add_extension("version", "1.2.3");
        pbo.add_file("test.txt", Cursor::new("test".as_bytes().to_vec()))
            .unwrap();
        let mut buffer = Vec::new();
        pbo.write(&mut Cursor::new(&mut buffer)).unwrap();
        assert_eq!(
            pbo.checksum().unwrap(),
            vec![
                32, 124, 152, 177, 225, 36, 144, 241, 249, 65, 204, 102, 179, 29, 72, 181, 149, 17,
                171, 46
            ]
        );
    }

    #[test]
    fn remove() {
        let mut pbo = WritablePbo::<Cursor<Vec<u8>>>::new();
        pbo.add_extension("prefix", "foobar");
        pbo.add_extension("version", "1.2.3");
        pbo.add_extension("remove_me", "faz");
        pbo.remove_extension("remove_me");
        pbo.add_file("test.txt", Cursor::new("test".as_bytes().to_vec()))
            .unwrap();
        pbo.add_file("test2.txt", Cursor::new("test".as_bytes().to_vec()))
            .unwrap();
        pbo.remove_file("test2.txt");
        let mut buffer = Vec::new();
        pbo.write(&mut Cursor::new(&mut buffer)).unwrap();
        assert_eq!(
            pbo.checksum().unwrap(),
            vec![
                32, 124, 152, 177, 225, 36, 144, 241, 249, 65, 204, 102, 179, 29, 72, 181, 149, 17,
                171, 46
            ]
        );
    }
}
