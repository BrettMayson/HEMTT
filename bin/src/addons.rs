use std::{fs::DirEntry, path::PathBuf, str::FromStr};

use hemtt_common::prefix::{self, Prefix};
use hemtt_project::AddonConfig;

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Addon {
    name: String,
    location: Location,
    config: Option<AddonConfig>,
    prefix: Prefix,
}

impl Addon {
    /// Create a new addon
    ///
    /// # Errors
    /// - [`Error::AddonPrefixMissing`] if the prefix is missing
    /// - [`std::io::Error`] if the addon.toml file cannot be read
    /// - [`toml::de::Error`] if the addon.toml file is invalid
    /// - [`std::io::Error`] if the prefix file cannot be read
    pub fn new(name: String, location: Location) -> Result<Self, Error> {
        let path = PathBuf::from(location.to_string()).join(&name);
        Ok(Self {
            config: {
                let path = path.join("addon.toml");
                if path.exists() {
                    Some(AddonConfig::from_file(&path)?)
                } else {
                    None
                }
            },
            prefix: {
                let mut prefix = None;
                let mut files = prefix::FILES
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>();
                files.append(
                    &mut prefix::FILES
                        .iter()
                        .map(|f| f.to_uppercase())
                        .collect::<Vec<String>>(),
                );
                'search: for file in &files {
                    let path = path.join(file);
                    if path.exists() {
                        let content = std::fs::read_to_string(path)?;
                        prefix = Some(Prefix::new(&content)?);
                        break 'search;
                    }
                }
                prefix.ok_or_else(|| Error::AddonPrefixMissing(name.clone()))?
            },
            location,
            name,
        })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn pbo_name(&self, prefix: &str) -> String {
        format!("{prefix}_{}", self.name)
    }

    #[must_use]
    pub const fn location(&self) -> &Location {
        &self.location
    }

    #[must_use]
    pub const fn prefix(&self) -> &Prefix {
        &self.prefix
    }

    #[must_use]
    /// addons/foobar
    /// optionals/foobar
    pub fn folder(&self) -> String {
        format!("{}/{}", self.location.to_string(), self.name)
    }

    #[must_use]
    pub const fn config(&self) -> Option<&AddonConfig> {
        self.config.as_ref()
    }

    /// Scan for addons in both locations
    ///
    /// # Errors
    /// - [`Error::AddonLocationInvalid`] if a location is invalid
    /// - [`Error::AddonLocationInvalid`] if a folder name is invalid
    pub fn scan() -> Result<Vec<Self>, Error> {
        let mut addons = Vec::new();
        for location in [Location::Addons, Location::Optionals] {
            addons.extend(location.scan()?);
        }
        Ok(addons)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    Addons,
    Optionals,
}

impl Location {
    /// Scan for addons in this location
    ///
    /// # Errors
    /// - [`Error::AddonLocationInvalid`] if a folder name is invalid
    pub fn scan(self) -> Result<Vec<Addon>, Error> {
        if !PathBuf::from(self.to_string()).exists() {
            return Ok(Vec::new());
        }
        std::fs::read_dir(self.to_string())?
            .collect::<std::io::Result<Vec<DirEntry>>>()?
            .iter()
            .map(std::fs::DirEntry::path)
            .filter(|file_or_dir| file_or_dir.is_dir())
            .map(|file| {
                let Some(name) = file.file_name() else {
                    return Err(Error::AddonLocationInvalid(file.display().to_string()));
                };
                let Some(name) = name.to_str() else {
                    return Err(Error::AddonLocationInvalid(file.display().to_string()));
                };
                Addon::new(name.to_string(), self)
            })
            .collect()
    }
}

impl FromStr for Location {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "addons" => Ok(Self::Addons),
            "optionals" => Ok(Self::Optionals),
            _ => Err(Error::AddonLocationInvalid(s.to_string())),
        }
    }
}

impl ToString for Location {
    fn to_string(&self) -> String {
        match self {
            Self::Addons => "addons",
            Self::Optionals => "optionals",
        }
        .to_string()
    }
}
