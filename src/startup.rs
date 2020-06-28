use std::io::Read;
use std::path::{Path, PathBuf};

use glob::glob;
use toml::Value::Table;

use crate::error::PrintableError;
use crate::HEMTTError;

macro_rules! exec {
    ($c:expr) => {
        $c().unwrap_or_print();
    };
}

pub fn startup() {
    exec!(check_git_ignore);
    exec!(deprecated_json);
    exec!(deprecated_values);
}

/// Checks for the recommended items in a .gitignore
/// Display a warning if they are not found
fn check_git_ignore() -> Result<(), HEMTTError> {
    if Path::new(".gitignore").exists() {
        let mut data = String::new();
        open_file!(".gitignore")?.read_to_string(&mut data)?;
        let mut ignore = crate::GIT_IGNORE.to_vec();
        for l in data.lines() {
            if let Some(index) = ignore.iter().position(|&d| d == l) {
                ignore.remove(index);
            }
        }
        for i in ignore {
            warn!(".gitignore is missing recommended value `{}`", i)
        }
    }
    Ok(())
}

fn deprecated_json() -> Result<(), HEMTTError> {
    if Path::new("hemtt.json").exists() {
        warn!("Use of `hemtt.json` is deprecated and may be removed in a future version, use `hemtt.toml`");
    }
    Ok(())
}

fn deprecated_values() -> Result<(), HEMTTError> {
    fn _check(file: PathBuf) -> Result<(), HEMTTError> {
        let items = [
            ("sig_name", "authority"),
            ("signame", "authority"),
            ("keyname", "key_name"),
            ("sigversion", "sig_version"),
            ("headerexts", "header_exts"),
        ];
        let mut data = String::new();
        open_file!(file)?.read_to_string(&mut data)?;
        for line in data.lines() {
            let value = line.parse::<toml::Value>();
            if let Ok(val) = value {
                if let Table(t) = val {
                    let old = items.iter().find(|x| t.contains_key((**x).0));
                    if let Some(o) = old {
                        warn!("deprecated value `{}` in `{}` - use `{}`", o.0, file.display(), o.1)
                    }
                }
            }
        }
        Ok(())
    }
    if Path::new("hemtt.toml").exists() {
        _check(PathBuf::from("hemtt.toml"))?;
    } else {
        for entry in glob("./.hemtt/*.toml").expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => _check(path)?,
                Err(e) => error!("{:?}", e),
            }
        }
    }
    Ok(())
}
