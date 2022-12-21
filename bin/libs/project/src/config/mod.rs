use std::{collections::HashMap, mem::MaybeUninit, path::Path, str::FromStr};

use hemtt_version::Version;
use serde::{Deserialize, Serialize};

use crate::{hemtt::Features, Error};

mod signing;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    /// The name of the project
    name: String,
    /// Prefix for the project
    prefix: String,

    /// Semver version of the project
    version: Option<String>,

    #[serde(default)]
    /// Headers to be added to built PBOs
    headers: HashMap<String, String>,

    #[serde(default)]
    /// Files to be included in the root of the project, supports glob patterns
    files: Vec<String>,

    #[serde(default)]
    hemtt: Features,

    #[serde(default)]
    signing: signing::Options,
}

impl Configuration {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the version of the project
    ///
    /// # Errors
    ///
    /// Returns an error if the version is not a valid semver version
    /// or a points to a file that does not contain a valid version macro
    pub fn version(&self) -> Result<Version, Error> {
        static mut VERSION: MaybeUninit<Version> = MaybeUninit::uninit();
        static mut INIT: bool = false;

        let version = self
            .version
            .as_ref()
            .map_or("script_version", |v| v.as_str());

        // Check for a cached version
        unsafe {
            if INIT {
                return Ok(VERSION.assume_init_ref().clone());
            }
        }

        // Check for script_version.hpp in the main addon
        let binding = if version == "script_version" {
            String::from("addons/main/script_version.hpp")
        } else {
            version.replace('\\', "/")
        };
        let path = Path::new(&binding);

        // Check for a path to a version macro file
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let version = Version::try_from_script_version(&content)?;
            unsafe {
                VERSION = MaybeUninit::new(version);
                INIT = true;
            }
            return unsafe { Ok(VERSION.assume_init_ref().clone()) };
        }

        Version::try_from(version).map_err(std::convert::Into::into)
    }

    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    #[must_use]
    pub const fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    #[must_use]
    pub fn files(&self) -> Vec<String> {
        let mut files = self.files.clone();
        for default in [
            "mod.cpp",
            "meta.cpp",
            "LICENSE",
            "logo_ca.paa",
            "logo_co.paa",
        ]
        .iter()
        .map(std::string::ToString::to_string)
        {
            if !files.contains(&default) {
                let path = Path::new(&default);
                if path.exists() {
                    files.push(default.clone());
                }
            }
        }
        files.sort();
        files.dedup();
        files
    }

    #[must_use]
    pub const fn hemtt(&self) -> &Features {
        &self.hemtt
    }

    #[must_use]
    pub const fn signing(&self) -> &signing::Options {
        &self.signing
    }

    /// Load a configuration from a file.
    ///
    /// # Errors
    ///
    /// If the file cannot be read, or if the file is not valid TOML, or if the
    /// file does not contain a valid configuration, an error is returned.
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let file = std::fs::read_to_string(path)?;
        Self::from_str(&file)
    }
}

impl FromStr for Configuration {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(Error::from)
    }
}
