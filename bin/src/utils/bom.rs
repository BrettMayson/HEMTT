use std::io::{Read, Seek, Write};

use crate::Error;

#[derive(clap::Parser)]
/// Convert UTF-8 with BOM to UTF-8 without BOM
pub struct Command {}

const ALLOWED_EXTENSIONS: [&str; 6] = ["sqf", "hpp", "cpp", "rvmat", "ext", "xml"];

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
        if path.is_file() {
            let ext = path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            if ALLOWED_EXTENSIONS.contains(&ext) {
                let mut file = std::fs::OpenOptions::new()
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
                    file.write_all(&buf)?;
                    info!("Removed BOM from {}", path.display());
                    count += 1;
                }
            } else {
                debug!("Skipping {}", path.display());
            }
        }
    }

    info!("Removed BOM from {} files", count);

    Ok(())
}
