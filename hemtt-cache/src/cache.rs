use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::{FileCacheGuard, Temporary};

#[derive(Copy, Clone)]
enum Lock {
    Unlocked,
    Locked,
    DNE,
}

#[derive(Default)]
pub struct FileCache {
    data: Arc<UnsafeCell<HashMap<PathBuf, Temporary>>>,
    locks: RwLock<HashMap<PathBuf, Lock>>,
}

unsafe impl Sync for FileCache {}

impl FileCache {
    pub(crate) fn unlock(&self, path: &PathBuf) {
        let mut locks = self.locks.write().unwrap();
        let lock = locks.get_mut(path).unwrap();
        match lock {
            Lock::Locked => *lock = Lock::Unlocked,
            Lock::Unlocked => {
                *lock = {
                    trace!("unlock was called on an unlocked path!");
                    Lock::Unlocked
                }
            }
            Lock::DNE => {
                trace!("unlock was called on an invalid path!");
            }
        }
    }

    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            data: Arc::new(UnsafeCell::new(HashMap::new())),
            locks: RwLock::new(HashMap::new()),
        }
    }

    /// Get a borrowed temporary
    ///
    /// Arguments:
    /// * `path`: PathBuf to the file, does not need to exist
    pub fn get<P: Into<PathBuf>>(&self, path: P) -> Option<FileCacheGuard> {
        let path = path.into();
        loop {
            let state = if let Some(lock) = self.locks.read().unwrap().get(&path) {
                *lock
            } else {
                Lock::DNE
            };
            if let Lock::Unlocked = state {
                let mut locks = self.locks.write().unwrap();
                let lock = locks.get_mut(&path).unwrap();
                match lock {
                    Lock::Locked => return None,
                    Lock::Unlocked => *lock = Lock::Locked,
                    Lock::DNE => return None,
                }
                return Some(FileCacheGuard {
                    inner: &self,
                    data: unsafe { (*self.data.get()).get_mut(&path).unwrap() },
                    path: path.to_owned(),
                });
            }
        }
    }

    /// Get a mutable borrowed temporary
    ///
    /// Arguments:
    /// * `path`: PathBuf to the file, does not need to exist
    pub fn get_mut<P: Into<PathBuf>>(&self, path: P) -> Option<FileCacheGuard> {
        let path = path.into();
        loop {
            let state = if let Some(lock) = self.locks.read().unwrap().get(&path) {
                *lock
            } else {
                Lock::DNE
            };
            if let Lock::Unlocked = state {
                let mut locks = self.locks.write().unwrap();
                let lock = locks.get_mut(&path).unwrap();
                match lock {
                    Lock::Locked => return None,
                    Lock::Unlocked => *lock = Lock::Locked,
                    Lock::DNE => return None,
                }
                return Some(FileCacheGuard {
                    inner: &self,
                    data: unsafe { (*self.data.get()).get_mut(&path).unwrap() },
                    path: path.to_owned(),
                });
            }
        }
    }

    /// Check if a path exists in the cache and on disk
    ///
    /// Arguments:
    /// * `path`: PathBuf to the file, does not need to exist
    pub fn exists<P: Into<PathBuf>>(&self, path: P) -> (bool, bool) {
        let path = path.into();
        (
            self.locks.read().unwrap().contains_key(&path),
            path.exists(),
        )
    }

    /// Get a mutable borrow of a temporary
    /// It will be read from disk if the path is not in the cache
    ///
    /// Arguments:
    /// * `path`: PathBuf to the file, does not need to exist
    pub fn read<P: Into<PathBuf>>(&self, path: P) -> Result<FileCacheGuard, std::io::Error> {
        let path = path.into();
        if !self.exists(&path).0 {
            trace!("Not in cache, retrieving from disk: `{:?}`", &path);
            self.insert(&path, Temporary::from_path(&path)?);
            // self.0.get_mut(&path).unwrap()
        }
        Ok({
            let mut tmp = self.get(&path).unwrap();
            tmp.seek(SeekFrom::Start(0))?;
            tmp
        })
    }

    /// Insert a temporary object into the cache
    /// Returns the previous value if one existed
    ///
    /// Arguments:
    /// * `path`: PathBuf to the file, does not need to exist
    /// * `tmp`: Temporary file object
    pub fn insert<P: Into<PathBuf>>(&self, path: P, tmp: Temporary) -> Option<Temporary> {
        let path = path.into();
        loop {
            let state = if let Some(lock) = self.locks.read().unwrap().get(&path) {
                *lock
            } else {
                Lock::DNE
            };
            match state {
                Lock::Unlocked | Lock::DNE => {
                    let mut locks = self.locks.write().unwrap();
                    let lock = locks.get_mut(&path);
                    match lock {
                        Some(Lock::Locked) => panic!("Poisoned Cache Lock"),
                        Some(Lock::Unlocked) => *(lock.unwrap()) = Lock::Locked,
                        _ => {}
                    }
                    locks.insert(path.to_owned(), Lock::Unlocked);
                    unsafe { return (&mut *self.data.get()).insert(path.to_owned(), tmp) }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::{Read, Write};
    use std::path::PathBuf;

    #[test]
    fn insert_retrieve() {
        let cache = super::FileCache::new();
        let path = PathBuf::from("./test.txt");
        cache.insert(path.clone(), {
            let mut tmp = super::Temporary::new();
            tmp.write_all(b"some text").unwrap();
            tmp.flush().unwrap();
            tmp
        });
        let mut tmp = cache.read(&path).unwrap();
        let mut buf = [0; 4];
        tmp.read_exact(&mut buf).unwrap();
        assert_eq!(b"some", &buf);
    }

    #[test]
    fn disk_retrieve() {
        let cache = super::FileCache::new();
        let mut tmp = cache.read(&PathBuf::from("./tests/text.txt")).unwrap();
        let mut buf = [0; 4];
        tmp.read_exact(&mut buf).unwrap();
        assert_eq!(b"This", &buf);
    }

    #[test]
    fn disk_multiple_retrieve() {
        let cache = super::FileCache::new();
        let mut tmp = cache.read(&PathBuf::from("./tests/text.txt")).unwrap();
        let mut buf = [0; 4];
        tmp.read_exact(&mut buf).unwrap();
        assert_eq!(b"This", &buf);
        let mut tmp = cache.read(&PathBuf::from("./tests/text2.txt")).unwrap();
        let mut buf = [0; 12];
        tmp.read_exact(&mut buf).unwrap();
        assert_eq!(b"This is also", &buf);
    }

    #[test]
    fn lock_release() {
        let cache = super::FileCache::new();
        {
            let mut tmp = cache.read(&PathBuf::from("./tests/text2.txt")).unwrap();
            let mut buf = [0; 12];
            tmp.read_exact(&mut buf).unwrap();
            assert_eq!(b"This is also", &buf);
        }
        {
            let mut tmp = cache.read(&PathBuf::from("./tests/text2.txt")).unwrap();
            let mut buf = [0; 12];
            tmp.read_exact(&mut buf).unwrap();
            assert_eq!(b"This is also", &buf);
        }
    }
}
