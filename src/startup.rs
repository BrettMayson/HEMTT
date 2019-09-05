use std::io::Read;
use std::path::Path;

use crate::error::PrintableError;
use crate::HEMTTError;

macro_rules! exec {
    ($c:expr) => {
        std::thread::spawn(|| {
            $c().unwrap_or_print();
        })
    };
}

pub fn startup() {
    exec!(check_git_ignore);
    exec!(deprecated);
}

/// Checks for the recommended items in a .gitignore
/// Display a warning if they are not found
fn check_git_ignore() -> Result<(), HEMTTError> {
    if Path::new(".gitignore").exists() {
        let mut data = String::new();
        open_file!(".gitignore")?.read_to_string(&mut data)?;
        let mut ignore = vec!["releases/*", "*.biprivatekey"];
        for l in data.lines() {
            if let Some(index) = ignore.iter().position(|&d| d == l) {
                ignore.remove(index);
            }
        }
        for i in ignore {
            warn!(format!(".gitignore is missing recommended value `{}`", i))
        }
    }
    Ok(())
}

fn deprecated() -> Result<(), HEMTTError> {
    if Path::new("hemtt.json").exists() {
        warnmessage!(
            "Use of `hemtt.json` is deprecated and may be removed in a future version",
            "Use `hemtt.toml` or a `.hemtt` project folder"
        );
    }
    Ok(())
}
