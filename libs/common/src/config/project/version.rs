use std::{mem::MaybeUninit, sync::RwLock};

use git2::Repository;
use serde::{Deserialize, Serialize};
use tracing::trace;
use vfs::VfsPath;

use crate::{error::Error, version::Version};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionConfig {
    path: String,
    defined: Option<(u32, u32, u32, Option<u32>)>,
    git_hash: u8,
}

static VERSION: RwLock<MaybeUninit<Version>> = RwLock::new(MaybeUninit::uninit());
static mut INIT: bool = false;

impl VersionConfig {
    /// Get the version of the project
    ///
    /// # Errors
    ///
    /// Returns an error if the version is not a valid semver version
    /// or a points to a file that does not contain a valid version macro
    pub fn get(&self, vfs: &VfsPath) -> Result<Version, Error> {
        // Check for a cached version
        unsafe {
            if INIT {
                return Ok(VERSION
                    .read()
                    .expect("VERSION poisoned")
                    .assume_init_ref()
                    .clone());
            }
        }

        let mut version = self.internal_get(vfs)?;

        if let Some(length) = self.git_hash() {
            let repo = Repository::discover(".")?;
            let rev = repo.revparse_single("HEAD")?;
            let id = rev.id().to_string();
            version.set_build(&id[0..length as usize]);
        }

        unsafe {
            *VERSION.write().expect("VERSION poisoned") = MaybeUninit::new(version.clone());
            INIT = true;
            Ok(VERSION
                .read()
                .expect("VERSION poisoned")
                .assume_init_ref()
                .clone())
        }
    }

    /// Invalidate the cached version
    #[allow(clippy::unused_self)]
    pub fn invalidate(&self) {
        unsafe {
            INIT = false;
        }
    }

    fn internal_get(&self, vfs: &VfsPath) -> Result<Version, Error> {
        // Check for a defined major version
        if let Some((major, minor, patch, build)) = self.defined {
            trace!("reading version from project.toml");
            return Ok(Version::new(major, minor, patch, build));
        }

        // Check for a path to a version macro file
        let path = vfs.join(&self.path)?;
        if path.exists()? {
            trace!("checking for version macro in {:?}", path);
            let content = path.read_to_string()?;
            return Version::try_from_script_version(&content).map_err(Into::into);
        }

        Err(crate::version::Error::UnknownVersion.into())
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    /// Length of the git hash to use in the build number
    ///
    /// Returns [`None`] if the length is 0
    pub const fn git_hash(&self) -> Option<u8> {
        if self.git_hash == 0 {
            None
        } else {
            Some(self.git_hash)
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct VersionSectionFile {
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

impl TryFrom<VersionSectionFile> for VersionConfig {
    type Error = Error;
    fn try_from(file: VersionSectionFile) -> Result<Self, Self::Error> {
        if (file.major.is_some() || file.minor.is_some() || file.patch.is_some())
            && file.path.is_some()
        {
            return Err(Error::Version(crate::version::Error::VersionPathConflict));
        }
        if file.major.is_none() && (file.minor.is_some() || file.patch.is_some()) {
            return Err(Error::Version(crate::version::Error::ExpectedMajor));
        }
        Ok(Self {
            path: file
                .path
                .unwrap_or_else(|| "addons/main/script_version.hpp".to_string())
                .replace('\\', "/"),
            defined: file
                .major
                .map(|major| {
                    Ok((
                        major,
                        file.minor.ok_or(crate::version::Error::ExpectedMinor)?,
                        file.patch.ok_or(crate::version::Error::ExpectedPatch)?,
                        file.build,
                    ))
                })
                .transpose()
                .map_err(Error::Version)?,
            git_hash: file.git_hash.unwrap_or(8),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_defined() {
        let toml = "
major = 1
minor = 2
patch = 3
build = 4
git_hash = 4
";
        let file: VersionSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = VersionConfig::try_from(file).expect("failed to convert");
        assert_eq!(config.path(), "addons/main/script_version.hpp");
        assert_eq!(config.git_hash(), Some(4));
    }

    #[test]
    fn path() {
        let toml = r#"
path = "test.hpp"
"#;
        let file: VersionSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = VersionConfig::try_from(file).expect("failed to convert");
        assert_eq!(config.path(), "test.hpp");
    }

    #[test]
    fn git_hash() {
        let toml = "
git_hash = 0
";
        let file: VersionSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = VersionConfig::try_from(file).expect("failed to convert");
        assert_eq!(config.git_hash(), None);
    }

    #[test]
    fn missing_major() {
        let toml = "
minor = 2
patch = 3
";
        let file: VersionSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = VersionConfig::try_from(file);
        assert!(config.is_err());
    }

    #[test]
    fn missing_minor() {
        let toml = "
major = 1
patch = 3
";
        let file: VersionSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = VersionConfig::try_from(file);
        assert!(config.is_err());
    }

    #[test]
    fn missing_patch() {
        let toml = "
major = 1
minor = 2
";
        let file: VersionSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = VersionConfig::try_from(file);
        assert!(config.is_err());
    }

    #[test]
    fn path_conflict() {
        let toml = r#"
path = "test.hpp"
major = 1
minor = 2
patch = 3
"#;
        let file: VersionSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = VersionConfig::try_from(file);
        assert!(config.is_err());
    }
}
