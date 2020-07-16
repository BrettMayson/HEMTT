use std::fs::{read_dir, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf, MAIN_SEPARATOR};

use crate::HEMTTError;

use crate::aerror;

pub fn read_prefix(prefix_path: &Path) -> String {
    let mut content = String::new();
    File::open(prefix_path)
        .unwrap()
        .read_to_string(&mut content)
        .unwrap();

    content.lines().nth(0).unwrap().to_string()
}

pub fn matches_include_path(path: &PathBuf, include_path: &str) -> bool {
    let include_pathbuf = PathBuf::from(&include_path.replace("\\", &MAIN_SEPARATOR.to_string()));

    if path.file_name() != include_pathbuf.file_name() {
        return false;
    }

    for parent in path.ancestors() {
        if parent.is_file() {
            continue;
        }

        let prefixpath = parent.join("$PBOPREFIX$");
        if !prefixpath.is_file() {
            continue;
        }

        let mut prefix = read_prefix(&prefixpath);

        prefix = if !prefix.is_empty() && prefix.chars().nth(0).unwrap() != '\\' {
            format!("\\{}", prefix)
        } else {
            prefix
        };
        let prefix_pathbuf = PathBuf::from(prefix.replace("\\", &MAIN_SEPARATOR.to_string()));

        let relative = path.strip_prefix(parent).unwrap();
        let test_path = prefix_pathbuf.join(relative);

        if test_path == include_pathbuf {
            return true;
        }
    }

    false
}

pub fn search_directory(include_path: &str, directory: PathBuf) -> Option<PathBuf> {
    for entry in read_dir(&directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().unwrap() == ".git" {
                continue;
            }

            if let Some(path) = search_directory(include_path, path) {
                return Some(path);
            }
        } else if matches_include_path(&path, include_path) {
            return Some(path);
        }
    }

    let direct_path = (&directory).to_str().unwrap().to_string()
        + &include_path.replace("\\", &MAIN_SEPARATOR.to_string());
    let direct_pathbuf = PathBuf::from(direct_path);

    if direct_pathbuf.is_file() {
        return Some(direct_pathbuf);
    }

    None
}

pub fn canonicalize(path: PathBuf) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            _ => {
                result.push(component);
            }
        }
    }
    result
}

pub fn find_include_file(
    include_path: &str,
    origin: Option<&PathBuf>,
    search_paths: &[PathBuf],
) -> Result<PathBuf, HEMTTError> {
    if include_path.chars().nth(0).unwrap() != '\\' {
        let mut path = PathBuf::from(include_path.replace("\\", &MAIN_SEPARATOR.to_string()));

        if let Some(origin_path) = origin {
            let absolute = PathBuf::from(&origin_path).canonicalize()?;
            let origin_dir = absolute.parent().unwrap();
            path = origin_dir.join(path);
        } else {
            path = std::env::current_dir()?.join(path);
        }

        let absolute = canonicalize(path);

        if !absolute.is_file() {
            match origin {
                Some(origin_path) => Err(aerror!(
                    "File \"{}\" included from \"{}\" not found.",
                    include_path,
                    origin_path.to_str().unwrap().to_string()
                )),
                None => Err(aerror!("Included file \"{}\" not found.", include_path)),
            }
        } else {
            Ok(absolute)
        }
    } else {
        for search_path in search_paths {
            if let Some(file_path) = search_directory(include_path, search_path.canonicalize()?) {
                return Ok(file_path);
            }
        }

        match origin {
            Some(origin_path) => Err(aerror!(
                "File \"{}\" included from \"{}\" not found.",
                include_path,
                origin_path.to_str().unwrap().to_string()
            )),
            None => Err(aerror!("Included file \"{}\" not found.", include_path)),
        }
    }
}
