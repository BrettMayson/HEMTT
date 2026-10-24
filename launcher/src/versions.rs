use fs_err;
use std::path::PathBuf;

use anyhow::Result;
use semver::Version;

/// Folder where the versions are downloaded
pub fn install_dir() -> PathBuf {
    let mut dir = dirs::data_local_dir().expect("Failed to get home directory");
    dir.push("hemtt-versions");
    dir
}

pub fn version_dir(version: &Version) -> PathBuf {
    let mut dir = install_dir();
    dir.push(version.to_string());
    dir
}

pub fn installed() -> Vec<Version> {
    let mut versions = Vec::new();
    if let Ok(entries) = fs_err::read_dir(install_dir()) {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string()
                && let Ok(version) = Version::parse(&file_name)
            {
                versions.push(version);
            }
        }
    }
    versions
}

pub fn remove(version: &str) -> Result<bool> {
    if let Ok(version) = Version::parse(version) {
        let dir = version_dir(&version);
        if dir.is_dir() {
            fs_err::remove_dir_all(&dir)?;
            return Ok(true);
        }
    }
    Ok(false)
}
