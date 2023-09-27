use std::{mem::MaybeUninit, path::Path};

use git2::Repository;
use hemtt_common::version::Version;
use serde::{Deserialize, Serialize};
use vfs::VfsPath;

use crate::error::Error;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Options {
    #[serde(default)]
    path: Option<String>,

    #[serde(default)]
    major: Option<u32>,
    #[serde(default)]
    minor: Option<u32>,
    #[serde(default)]
    patch: Option<u32>,
    #[serde(default)]
    build: Option<u32>,

    #[serde(default)]
    git_hash: Option<u8>,
}

static mut VERSION: MaybeUninit<Version> = MaybeUninit::uninit();
static mut INIT: bool = false;

impl Options {
    /// Get the version of the project
    ///
    /// # Errors
    ///
    /// Returns an error if the version is not a valid semver version
    /// or a points to a file that does not contain a valid version macro
    pub fn get(&self, vfs: Option<&VfsPath>) -> Result<Version, Error> {
        // Check for a cached version
        unsafe {
            if INIT {
                return Ok(VERSION.assume_init_ref().clone());
            }
        }

        let mut version = self._get(vfs)?;

        if let Some(length) = self.git_hash() {
            let repo = Repository::discover(".")?;
            let rev = repo.revparse_single("HEAD")?;
            let id = rev.id().to_string();
            version.set_build(&id[0..length as usize]);
        };

        unsafe {
            VERSION = MaybeUninit::new(version);
            INIT = true;
            return Ok(VERSION.assume_init_ref().clone());
        }
    }

    /// Invalidate the cached version
    #[allow(clippy::unused_self)]
    pub fn invalidate(&self) {
        unsafe {
            INIT = false;
        }
    }

    fn _get(&self, vfs: Option<&VfsPath>) -> Result<Version, Error> {
        // Check for a defined major version
        if let Some(major) = self.major {
            trace!("reading version from project.toml");
            let Some(minor) = self.minor else {
                return Err(hemtt_common::version::Error::ExpectedMinor.into());
            };
            let Some(patch) = self.patch else {
                return Err(hemtt_common::version::Error::ExpectedPatch.into());
            };
            return Ok(Version::new(major, minor, patch, self.build));
        }

        let version = self.path.as_ref().map_or("script_version", |v| v.as_str());

        // Check for script_version.hpp in the main addon
        let binding = if version == "script_version" {
            String::from("addons/main/script_version.hpp")
        } else {
            version.replace('\\', "/")
        };
        let path = Path::new(&binding);

        // Check for a path to a version macro file
        if path.exists() {
            trace!("checking for version macro in {:?}", path);
            let content = if let Some(vfs) = vfs {
                vfs.join(&binding)?.read_to_string()?
            } else {
                std::fs::read_to_string(path)?
            };
            return Version::try_from_script_version(&content).map_err(Into::into);
        }
        error!("could not find version macro file: {:?}", path);

        Err(hemtt_common::version::Error::UnknownVersion.into())
    }

    #[must_use]
    pub const fn git_hash(&self) -> Option<u8> {
        if let Some(include) = self.git_hash {
            if include == 0 {
                return None;
            }
            Some(include)
        } else {
            Some(8)
        }
    }
}
