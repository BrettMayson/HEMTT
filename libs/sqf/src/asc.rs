use std::{fs::File, io::Write, path::Path};

use hemtt_common::error::thiserror;
use rust_embed::RustEmbed;
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
