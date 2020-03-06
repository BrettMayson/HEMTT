use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::{HEMTTError, IOPathError};

#[derive(Debug, Default)]
pub struct FileCache {
    files: HashMap<String, Vec<u8>>,
    root: String,
}

impl FileCache {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            root: {
                let mut current = std::env::current_dir().unwrap().to_str().unwrap().replace("\\\\", "\\");
                current.push('\\');
                current
            },
        }
    }

    fn clean_path(&self, path: &str) -> String {
        path.replace("\\\\?\\", "").replace(&self.root, "")
    }

    #[allow(clippy::map_entry)]
    pub fn read(&mut self, path: &str) -> Result<Vec<u8>, HEMTTError> {
        let path = self.clean_path(path);
        if self.files.contains_key(&path) {
            Ok(self.files.get(&path).unwrap().to_vec())
        } else {
            let f = open_file!(path)?;
            let mut reader = BufReader::new(f);
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).map_err(|e| {
                HEMTTError::PATH(IOPathError {
                    source: e,
                    path: PathBuf::from(&path),
                })
            })?;
            self.files.insert(path, buf.clone());
            Ok(buf)
        }
    }

    // This should be fixed in armake2, this is a slow workaround but it works for now
    pub fn clean_comments(&mut self, path: &str) -> Result<String, std::io::Error> {
        // if !PathBuf::from(path).exists() {
        //     return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "can't find that file eh"));
        // }
        let keep = Regex::new(r#"(?m)QUOTE\((.+?)\)|"([^"]+)"|"(.+)$"#).unwrap();
        let clean = Regex::new(r#"(?m)(?:(?://.+?)$)|(?:/\*(?:.+?)\*/)"#).unwrap();
        let content = self.as_string(path).unwrap().replace("\r\n", "\n");
        let mut safe = HashMap::new();
        for mat in keep.find_iter(&content) {
            safe.insert(mat.start(), mat.end());
        }
        let mut output = String::new();
        let mut cursor = 0;
        'outer: for mat in clean.find_iter(&content) {
            let mat_start = mat.start();
            let mat_end = mat.end();
            if mat_start == 0 {
                output.push_str(&content[cursor..mat_end]);
                break;
            }
            for (start, end) in safe.iter() {
                if *start <= (mat_start - 1) && *end > mat_start {
                    output.push_str(&content[cursor..mat_end]);
                    cursor = mat_end;
                    continue 'outer;
                }
            }
            if &content[(mat_start - 1)..mat_start] == "/" {
                output.push_str(&content[cursor..(mat_start - 1)]);
            } else {
                output.push_str(&content[cursor..(mat_start)]);
            }
            cursor = mat_end;
        }
        output.push_str(&content[cursor..(content.len())]);
        Ok(output)
    }

    pub fn as_string(&mut self, path: &str) -> Result<String, HEMTTError> {
        String::from_utf8(self.read(path)?).map_err(From::from)
    }

    pub fn lines(&mut self, path: &str) -> Result<Vec<String>, HEMTTError> {
        Ok(String::from_utf8(self.read(path)?)?.lines().map(|l| l.to_string()).collect())
    }

    pub fn insert(&mut self, path: &str, data: String) -> Result<(), HEMTTError> {
        debug!("Cache insert: `{}`", path);
        self.files.insert(path.to_string(), data.as_bytes().to_vec());
        Ok(())
    }

    pub fn insert_bytes(&mut self, path: &str, data: Vec<u8>) -> Result<(), HEMTTError> {
        debug!("Cache insert bytes: `{}`", path);
        self.files.insert(path.to_string(), data);
        Ok(())
    }

    pub fn get_line(&mut self, path: &str, line: usize) -> Result<String, HEMTTError> {
        Ok(self.lines(path)?[line - 1].clone())
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
        self.redirects.insert(original, tmp);
        Ok(())
    }

    pub fn get_path(&self, original: String) -> Option<&String> {
        self.redirects.get(&original)
    }

    /// Gets the rendered path from the original
    pub fn get_paths(&self, original: String) -> (String, String) {
        let rendered = crate::build::prebuild::render::can_render(&Path::new(&original));
        if rendered {
            (
                original.replace(".ht.", ".").trim_end_matches(".ht").to_string(),
                self.redirects.get(&original).unwrap().to_string(),
            )
        } else {
            (original.clone(), original)
        }
    }
}

impl RenderedFiles {
    pub fn clean(&mut self) {
        for (_, tmp) in self.redirects.iter() {
            if let Err(e) = remove_file!(tmp) {
                error!(e.to_string());
            }
        }
    }
}
