use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Provides a temporary write + read that is deleted from the disk when it is dropped
/// The data will only hit the disk if it is over 1Mb in size

#[derive(Debug, Default)]
pub struct Temporary {
    data: Vec<u8>,
    disk: Option<(PathBuf, File)>,
    pointer: usize,
    max_size: usize,
    cleanup: bool,
}

impl Temporary {
    /// Create a new temporary file with default options
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            disk: None,
            pointer: 0,
            max_size: 1_000_000,
            cleanup: true,
        }
    }

    /// Create a new temporary file with a max size in memory
    ///
    /// Arguments:
    /// * `max`: number of bytes to use in memory before dumping to disk
    pub fn new_with_max(max: usize) -> Self {
        let mut s = Self::new();
        s.max_size = max;
        s
    }

    /// Create a temporary file object from an existing file
    ///
    /// Arguments:
    /// * `path`: Path to existing file
    pub fn from_path<P: Into<PathBuf>>(path: P) -> std::io::Result<Self> {
        let path = path.into();
        let mut s = Self::new_with_max(0);
        s.disk = Some((path.clone(), Self::open_read_write(path)?));
        s.cleanup = false;
        Ok(s)
    }

    /// Create a temporary file object from a string
    ///
    /// Arguments:
    /// * `content`: Content to place in temporary value
    pub fn from_string(content: &str) -> std::io::Result<Self> {
        let mut tmp = Self::new();
        tmp.write_all(content.as_bytes())?;
        Ok(tmp)
    }

    fn open_read_write(path: PathBuf) -> std::io::Result<File> {
        Ok(std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?)
    }
}

impl std::io::Write for Temporary {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.pointer + buf.len() < self.max_size {
            for i in self.pointer..(self.pointer + buf.len()) {
                self.data.insert(i, buf[i - self.pointer]);
            }
            self.pointer += buf.len();
            Ok(buf.len())
        } else {
            if self.disk.is_none() {
                let mut root = std::env::temp_dir();
                root.push("hemtt_cache");
                if !root.exists() {
                    // Avoid race condition of multiple threads ending up here at the exact same time
                    if let Err(e) = std::fs::create_dir(&root) {
                        if !root.exists() {
                            return Err(e);
                        }
                    }
                }
                #[allow(unused_assignments)]
                let mut path = root.clone();
                while {
                    path = root.clone();
                    path.push(uuid::Uuid::new_v4().to_string().replace("-", ""));
                    path.exists()
                } {}
                trace!("Creating file at {:?}", path);
                self.disk = Some((path.clone(), Self::open_read_write(path)?));
                if let Some((_, file)) = &mut self.disk {
                    file.write_all(&self.data)?;
                }
                self.data = Vec::new();
            }
            if let Some((_, file)) = &mut self.disk {
                match file.write(buf) {
                    Ok(bytes) => {
                        self.pointer += bytes;
                        Ok(bytes)
                    }
                    Err(e) => Err(e),
                }
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "The disk should have been created, but it was not available",
                ))
            }
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl std::io::Read for Temporary {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some((_, file)) = &mut self.disk {
            match file.read(buf) {
                Ok(bytes) => {
                    self.pointer += bytes;
                    Ok(bytes)
                }
                Err(e) => Err(e),
            }
        } else {
            let max = std::cmp::min(self.data.len(), self.pointer + buf.len());
            for i in self.pointer..max {
                buf[i - self.pointer] = self.data[i];
            }
            let size = max - self.pointer;
            self.pointer = max;
            Ok(size)
        }
    }
}

impl std::io::Seek for Temporary {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let new = match pos {
            std::io::SeekFrom::Start(i) => i as i64,
            std::io::SeekFrom::Current(i) => self.pointer as i64 + i,
            std::io::SeekFrom::End(i) => self.pointer as i64 + i,
        };
        if new < 0 {
            Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Seeking to before the start",
            ))
        } else {
            if let Some((_, file)) = &mut self.disk {
                file.seek(pos)?;
            }
            self.pointer = new as usize;
            Ok(new as u64)
        }
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        if self.cleanup {
            if let Some((path, _)) = &self.disk {
                trace!("deleting file at drop: {:?}", path);
                if let Err(e) = std::fs::remove_file(path) {
                    trace!("failed to delete {:?}: {}", path, e);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::{Read, Seek, Write};
    // Memory Only
    #[test]
    fn memory_single_write_single_read() {
        let mut tmp = super::Temporary::new();
        tmp.write_all(b"some text").unwrap();
        tmp.flush().unwrap();
        tmp.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut buffer = [0; 9];
        tmp.read_exact(&mut buffer).unwrap();
        assert_eq!(b"some text", &buffer);
    }

    #[test]
    fn memory_multi_write_single_read() {
        let mut tmp = super::Temporary::new();
        tmp.write_all(b"some").unwrap();
        tmp.write_all(b" text").unwrap();
        tmp.flush().unwrap();
        tmp.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut buffer = [0; 9];
        tmp.read_exact(&mut buffer).unwrap();
        assert_eq!(b"some text", &buffer);
    }

    #[test]
    fn memory_single_write_multi_read() {
        let mut tmp = super::Temporary::new();
        tmp.write_all(b"some text").unwrap();
        tmp.flush().unwrap();
        tmp.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut buffer = [0; 4];
        tmp.read_exact(&mut buffer).unwrap();
        assert_eq!(b"some", &buffer);
        let mut buffer = [0; 5];
        tmp.read_exact(&mut buffer).unwrap();
        assert_eq!(b" text", &buffer);
    }

    #[test]
    fn memory_multi_write_multi_read() {
        let mut tmp = super::Temporary::new();
        tmp.write_all(b"some").unwrap();
        tmp.write_all(b" text").unwrap();
        tmp.flush().unwrap();
        tmp.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut buffer = [0; 4];
        tmp.read_exact(&mut buffer).unwrap();
        assert_eq!(b"some", &buffer);
        let mut buffer = [0; 5];
        tmp.read_exact(&mut buffer).unwrap();
        assert_eq!(b" text", &buffer);
    }

    // Disk
    #[test]
    fn disk_single_write_single_read() {
        let mut tmp = super::Temporary::new_with_max(2);
        tmp.write_all(b"some text").unwrap();
        tmp.flush().unwrap();
        tmp.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut buffer = [0; 9];
        tmp.read_exact(&mut buffer).unwrap();
        assert_eq!(b"some text", &buffer);
    }

    #[test]
    fn disk_multi_write_single_read() {
        let mut tmp = super::Temporary::new_with_max(2);
        tmp.write_all(b"some").unwrap();
        tmp.write_all(b" text").unwrap();
        tmp.flush().unwrap();
        tmp.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut buffer = [0; 9];
        tmp.read_exact(&mut buffer).unwrap();
        assert_eq!(b"some text", &buffer);
    }

    #[test]
    fn disk_single_write_multi_read() {
        let mut tmp = super::Temporary::new_with_max(2);
        tmp.write_all(b"some text").unwrap();
        tmp.flush().unwrap();
        tmp.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut buffer = [0; 4];
        tmp.read_exact(&mut buffer).unwrap();
        assert_eq!(b"some", &buffer);
        let mut buffer = [0; 5];
        tmp.read_exact(&mut buffer).unwrap();
        assert_eq!(b" text", &buffer);
    }

    #[test]
    fn disk_multi_write_multi_read() {
        let mut tmp = super::Temporary::new_with_max(2);
        tmp.write_all(b"some").unwrap();
        tmp.write_all(b" text").unwrap();
        tmp.flush().unwrap();
        tmp.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut buffer = [0; 4];
        tmp.read_exact(&mut buffer).unwrap();
        assert_eq!(b"some", &buffer);
        let mut buffer = [0; 5];
        tmp.read_exact(&mut buffer).unwrap();
        assert_eq!(b" text", &buffer);
    }
}
