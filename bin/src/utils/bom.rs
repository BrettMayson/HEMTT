use std::io::{Read, Seek, Write};

use crate::{Error, TEXT_EXTENSIONS};

#[derive(clap::Parser)]
/// Convert UTF-8 with BOM to UTF-8 without BOM
///
/// ## About BOM
///
/// A Byte Order Mark (BOM) is an invisible character at the start of text files
/// that can cause issues with Arma 3's config and script parsers.
///
/// This utility:
/// - Scans your project for files with BOM markers
/// - Removes them from supported file types (sqf, hpp, cpp, etc.)
/// - Reports how many files were modified
///
/// BOM markers are often added by Windows text editors. Run this utility if you
/// encounter unexpected parsing errors or as part of project maintenance.
pub struct Command {}

/// Execute the bom command
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
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        if !TEXT_EXTENSIONS.contains(&ext) {
            debug!("Skipping {}", path.display());
            continue;
        }
        let mut file = fs_err::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        let mut buf = [0; 3];
        if file.read_exact(&mut buf).is_err() {
            continue;
        }
        if buf == [0xEF, 0xBB, 0xBF] {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            file.seek(std::io::SeekFrom::Start(0))?;
            if let Err(e) = file.write_all(&buf) {
                error!("Failed to write to {}: {}", path.display(), e);
                continue;
            }
            file.set_len(buf.len() as u64)?;
            info!("Removed BOM from {}", path.display());
            count += 1;
        }
    }

    info!("Removed BOM from {} files", count);

    Ok(())
}
