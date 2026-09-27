//! Which files in an addon HEMTT config-checks.
//!
//! Shared so the CLI and the language server cannot disagree about it. They
//! did: the CLI checked every rapifiable file and honoured the addon's
//! `rapify` settings, while the editor only ever looked at `config.cpp` and
//! ignored `exclude` and `enabled` entirely.

use std::io::Read;

use hemtt_workspace::{WorkspacePath, addons::Addon};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Workspace error: {0}")]
    Workspace(#[from] hemtt_workspace::Error),
    #[error("Invalid rapify exclude pattern: {0}")]
    Pattern(#[from] glob::PatternError),
}

/// Extensions that HEMTT rapifies.
const EXTENSIONS: &[&str] = &["cpp", "rvmat", "ext", "sqm", "bikb", "bisurf"];

/// Can this file be rapified?
///
/// A file that is already rapified is skipped - it starts with `\0raP`.
///
/// # Errors
/// [`hemtt_workspace::Error`] if the file cannot be read
pub fn can_rapify(entry: &WorkspacePath) -> Result<bool, hemtt_workspace::Error> {
    let path = entry.as_str();
    let pathbuf = std::path::PathBuf::from(&path);
    let Some(ext) = pathbuf.extension().and_then(std::ffi::OsStr::to_str) else {
        return Ok(false);
    };
    if ext == "cpp" && pathbuf.file_name() != Some(std::ffi::OsStr::new("config.cpp")) {
        tracing::warn!(
            "{} - cpp files other than config.cpp are usually not intentional. use hpp for includes",
            path.trim_start_matches('/')
        );
    }
    if !EXTENSIONS.contains(&ext) {
        return Ok(false);
    }
    let mut buffer = vec![0; 4];
    if entry.open_file()?.read_exact(&mut buffer).is_err() {
        // The file is less than 4 bytes, so it is not rapified
        return Ok(true);
    }
    Ok(buffer != b"\0raP")
}

/// The files in an addon that should be config-checked.
///
/// Honours the addon's `rapify` settings: nothing if it is disabled, and
/// anything matching an `exclude` pattern is left out.
///
/// # Errors
/// [`Error::Workspace`] if the addon cannot be walked
/// [`Error::Pattern`] if an `exclude` pattern is not a valid glob
pub fn checkable(root: &WorkspacePath, addon: &Addon) -> Result<Vec<WorkspacePath>, Error> {
    let mut excludes = Vec::new();
    if let Some(config) = addon.config() {
        if !config.rapify().enabled() {
            tracing::debug!("rapify disabled for {}", addon.name());
            return Ok(Vec::new());
        }
        for pattern in config.rapify().exclude() {
            excludes.push(glob::Pattern::new(pattern)?);
        }
    }
    let options = glob::MatchOptions {
        require_literal_separator: true,
        ..Default::default()
    };
    let mut files = Vec::new();
    for entry in root.join(addon.folder())?.walk_dir()? {
        if !entry.is_file()? || !can_rapify(&entry)? {
            continue;
        }
        if excludes
            .iter()
            .any(|pattern| pattern.matches_with(entry.as_str(), options))
        {
            tracing::debug!("skipping {}", entry.as_str());
            continue;
        }
        files.push(entry);
    }
    Ok(files)
}
