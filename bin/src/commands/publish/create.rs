//! Storing a new published ID in `meta.cpp`

use crate::Error;

/// Write `meta.cpp` in the current directory with `published_id`, replacing the file
///
/// # Errors
/// If the file cannot be written
pub fn store_id(published_id: u64) -> Result<(), Error> {
    let content = format!("protocol = 1;\npublishedid = {published_id};");
    let meta_path = std::env::current_dir()?.join("meta.cpp");
    fs_err::write(meta_path, content)?;
    Ok(())
}
