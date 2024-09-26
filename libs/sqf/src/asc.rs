use std::{fs::File, io::Write, path::Path};

use hemtt_common::error::thiserror;
use rust_embed::RustEmbed;
use serde::Serialize;
use tracing::trace;

#[cfg(windows)]
#[derive(RustEmbed)]
#[folder = "dist/windows"]
struct Distributables;

#[cfg(not(windows))]
#[derive(RustEmbed)]
#[folder = "dist/linux"]
struct Distributables;

#[cfg(windows)]
const SOURCE: [&str; 1] = ["asc.exe"];

#[cfg(not(windows))]
const SOURCE: [&str; 1] = ["asc"];

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[must_use]
pub const fn command() -> &'static str {
    SOURCE[0]
}

/// Install Arma Script Compiler
///
/// # Errors
/// [`Error::Io`] if the file couldn't be created or written to
///
/// # Panics
/// If an expected file didn't get packed into the binary
pub fn install(path: &Path) -> Result<(), super::Error> {
    _install(path).map_err(super::Error::AscError)
}

fn _install(path: &Path) -> Result<(), Error> {
    let _ = std::fs::create_dir_all(path);
    for file in SOURCE {
        let out = path.join(file);
        trace!("unpacking {:?} to {:?}", file, out.display());
        let mut f = File::create(&out)?;
        f.write_all(
            &Distributables::get(file)
                .expect("dist files should exist")
                .data,
        )?;
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(out, PermissionsExt::from_mode(0o744))?;
        }
    }
    Ok(())
}

#[derive(Default, Serialize)]
pub struct ASCConfig {
    #[serde(rename = "inputDirs")]
    input_dirs: Vec<String>,
    #[serde(rename = "outputDir")]
    output_dir: String,
    #[serde(rename = "includePaths")]
    include_dirs: Vec<String>,
    #[serde(rename = "excludeList")]
    exclude_list: Vec<String>,
    #[serde(rename = "workerThreads")]
    worker_threads: usize,
}

impl ASCConfig {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            input_dirs: vec![],
            output_dir: String::new(),
            include_dirs: vec![],
            exclude_list: vec![],
            worker_threads: 2,
        }
    }

    pub fn add_input_dir(&mut self, dir: String) {
        if self.input_dirs.contains(&dir) {
            return;
        }
        self.input_dirs.push(dir);
    }

    pub fn set_output_dir(&mut self, dir: String) {
        self.output_dir = dir;
    }

    pub fn add_include_dir(&mut self, dir: String) {
        self.include_dirs.push(dir);
    }

    pub fn add_exclude(&mut self, dir: &str) {
        self.exclude_list.push(dir.replace('/', "\\"));
    }

    pub fn set_worker_threads(&mut self, threads: usize) {
        self.worker_threads = threads;
    }
}
