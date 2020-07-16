use std::collections::HashMap;
use std::path::PathBuf;
use std::io::{Seek, SeekFrom};

use crate::tmp::Temporary;

pub struct FileCache(HashMap<PathBuf, Temporary>);

impl FileCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Get a borrowed temporary
    ///
    /// Arguments:
    /// * `path`: PathBuf to the file, does not need to exist
    pub fn get(&mut self, path: &PathBuf) -> Option<&Temporary> {
        self.0.get(path)
    }

    /// Get a mutable borrowed temporary
    ///
    /// Arguments:
    /// * `path`: PathBuf to the file, does not need to exist
    pub fn get_mut(&mut self, path: &PathBuf) -> Option<&mut Temporary> {
        self.0.get_mut(path)
    }

    /// Check if a path exists in the cache and on disk
    ///
    /// Arguments:
    /// * `path`: PathBuf to the file, does not need to exist
    pub fn exists(&mut self, path: &PathBuf) -> (bool, bool) {
        (self.0.contains_key(path), path.exists())
    }

    /// Get a mutable borrow of a temporary
    /// It will be read from disk if the path is not in the cache
    ///
    /// Arguments:
    /// * `path`: PathBuf to the file, does not need to exist
    pub fn read(&mut self, path: &PathBuf) -> Result<&mut Temporary, std::io::Error> {
        Ok({
            let tmp = if self.0.contains_key(path) {
                    self.0.get_mut(path).unwrap()
                } else {
                    trace!("Not in cache, retrieving from disk: `{:?}`", path);
                    self.0
                        .insert(path.to_owned(), Temporary::from_path(path.to_owned())?);
                    self.0.get_mut(path).unwrap()
                };
            tmp.seek(SeekFrom::Start(0))?;
            tmp
        })
    }

    /// Insert a temporary object into the cache
    ///
    /// Arguments:
    /// * `path`: PathBuf to the file, does not need to exist
    /// * `tmp`: Temporary file object
    pub fn insert(&mut self, path: PathBuf, tmp: Temporary) -> Option<Temporary> {
        self.0.insert(path, tmp)
    }
}

#[cfg(test)]
mod test {
    use std::io::{Read, Write};
    use std::path::PathBuf;

    #[test]
    fn insert_retrieve() {
        let mut cache = super::FileCache::new();
        let path = PathBuf::from("./test.txt");
        cache.insert(path.clone(), {
            let mut tmp = super::Temporary::new();
            tmp.write(b"some text").unwrap();
            tmp.flush().unwrap();
            tmp
        });
        let tmp = cache.read(&path).unwrap();
        let mut buf = [0; 4];
        tmp.read(&mut buf).unwrap();
        assert_eq!(b"some", &buf);
    }

    #[test]
    fn disk_retrieve() {
        let mut cache = super::FileCache::new();
        let tmp = cache.read(&PathBuf::from("./tests/text.txt")).unwrap();
        let mut buf = [0; 4];
        tmp.read(&mut buf).unwrap();
        assert_eq!(b"This", &buf);
    }
}
