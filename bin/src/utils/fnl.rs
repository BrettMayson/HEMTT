use std::io::{Read, Seek, SeekFrom, Write};

use crate::{Error, modules::fnl::TEXT_EXTENSIONS};

#[derive(clap::Parser)]
/// Insert a final newline into files if missing
///
/// ## Why Final Newlines Matter
///
/// Many tools and standards (POSIX) expect text files to end with a newline character.
/// Missing final newlines can cause:
/// - Git diff warnings
/// - Issues with some text processing tools
/// - Inconsistent behavior across different editors
///
/// This utility:
/// - Scans text files (sqf, hpp, cpp, txt, etc.)
/// - Adds a final newline if missing
/// - Reports how many files were modified
///
/// Recommended to run before committing code to ensure consistency.
pub struct Command {}

/// Execute the final newline command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(_: &Command) -> Result<(), Error> {
    let mut count = 0;
    for entry in walkdir::WalkDir::new(".") {
        let entry = entry?;
        let path = entry.path();
        if path.display().to_string().contains(".hemttout") {
            continue;
        }
        if path.is_file() {
            let ext = path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            if TEXT_EXTENSIONS.contains(&ext) {
                let mut file = fs_err::OpenOptions::new()
                    .read(true)
                    .append(true)
                    .open(path)?;
                let len = file.seek(SeekFrom::End(0))?;
                if len == 0 {
                    file.write_all(b"\n")?;
                    info!("Inserted final newline into {}", path.display());
                    count += 1;
                    continue;
                }
                file.seek(SeekFrom::End(-1))?;
                let mut last_byte = [0u8; 1];
                file.read_exact(&mut last_byte)?;
                if last_byte != [b'\n'] {
                    file.write_all(b"\n")?;
                    info!("Inserted final newline into {}", path.display());
                    count += 1;
                }
            } else {
                debug!("Skipping {}", path.display());
            }
        }
    }

    info!("Inserted final newline into {} files", count);

    Ok(())
}
