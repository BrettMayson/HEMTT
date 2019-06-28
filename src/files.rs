use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader,Read};
use std::path::{Path, PathBuf};

use crate::{HEMTTError, IOPathError};

#[derive(Debug)]
pub struct FileCache {
    files: HashMap<String, Vec<u8>>,
}

impl FileCache {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn read(&mut self, path: &str) -> Result<Vec<u8>, HEMTTError> {
        if self.files.contains_key(path) {
            Ok(self.files.get(path).unwrap().to_vec())
        } else {
            let f = File::open(path)?;
            let mut reader = BufReader::new(f);
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).map_err(|e| HEMTTError::PATH(IOPathError {
                source: e,
                path: PathBuf::from(path),
            }))?;
            self.files.insert(path.to_string(), buf.clone());
            Ok(buf)
        }
    }

    pub fn as_string(&mut self, path: &str) -> Result<String, HEMTTError> {
        String::from_utf8(self.read(path)?).map_err(From::from)
    }

    pub fn lines(&mut self, path: &str) -> Result<Vec<String>, HEMTTError> {
        Ok(String::from_utf8(self.read(path)?)?.lines().map(|l| l.to_string()).collect())
    }

    pub fn insert(&mut self, path: &str, data: String) -> Result<(), HEMTTError> {
        self.files.insert(path.to_string(), data.as_bytes().to_vec());
        Ok(())
    }

    pub fn get_line(&mut self, path: &str, line: usize) -> Result<String, HEMTTError> {
        Ok(self.lines(path)?[line].clone())
    }
}

#[derive(Default, Clone)]
pub struct RenderedFiles {
    redirects: HashMap<String, String>,
    pub no_drop: bool,
}

impl RenderedFiles {
    pub fn new() -> Self {
        Self {
            redirects: HashMap::new(),
            no_drop: false,
        }
    }

    pub fn add(&mut self, original: String, tmp: String) -> Result<(), HEMTTError> {
        self.redirects.insert(original.clone(), tmp.clone());
        Ok(())
    }

    pub fn get_path(&self, original: String) -> Option<&String> {
        self.redirects.get(&original)
    }

    pub fn get_paths(&self, original: String) -> (String, String) {
        let rendered = crate::build::prebuild::render::can_render(&Path::new(&original));
        if rendered {
            (original.replace(".ht.", ".").trim_end_matches(".ht").to_string(), self.redirects.get(&original).unwrap().to_string())
        } else {
            (original.clone(), original)
        }
    }
}

impl RenderedFiles {
    pub fn clean(&mut self) {
        for (_, tmp) in self.redirects.iter() {
            if let Err(e) = std::fs::remove_file(tmp) {
                error!(e.to_string());
            }
        }
    }
}
