use std::collections::HashSet;
use std::fmt::Display;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::{fs::DirEntry, str::FromStr};

use hemtt_common::config::AddonConfig;
use hemtt_common::prefix::{FILES, Prefix};
use hemtt_common::version::Version;
use tracing::{trace, warn};

use crate::WorkspacePath;
use crate::position::Position;
use crate::reporting::{Code, Mapping};

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Addon duplicated with different case: {0}")]
    NameDuplicate(String),
    #[error("Addon present in addons and optionals: {0}")]
    Duplicate(String),
    #[error("Invalid addon location: {0}")]
    LocationInvalid(String),
    #[error("Optional addon not found: {0}")]
    OptionalNotFound(String),
    #[error("Addon prefix not found: {0}")]
    PrefixMissing(String),
}

#[derive(Debug, Clone)]
pub struct Addon {
    name: String,
    location: Location,
    config: Option<AddonConfig>,
    prefix: Prefix,
    build_data: BuildData,
}

impl Addon {
    /// Create a new addon
    ///
    /// # Errors
    /// - [`Error::AddonPrefixMissing`] if the prefix is missing
    /// - [`std::io::Error`] if the addon.toml file cannot be read
    /// - [`toml::de::Error`] if the addon.toml file is invalid
    /// - [`std::io::Error`] if the prefix file cannot be read
    pub fn new(root: &Path, name: String, location: Location) -> Result<Self, crate::error::Error> {
        let path = root.join(location.to_string()).join(&name);
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
                let mut files = FILES
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>();
                files.append(
                    &mut FILES
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
                prefix.ok_or_else(|| Error::PrefixMissing(name.clone()))?
            },
            location,
            name,
            build_data: BuildData::new(),
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
    /// The addon's path from the root
    /// addons/foobar
    /// optionals/foobar
    pub fn folder(&self) -> String {
        format!("{}/{}", self.location, self.name)
    }

    #[must_use]
    /// The addon's path from the root, as a `PathBuf`
    pub fn folder_pathbuf(&self) -> PathBuf {
        PathBuf::from(self.location.to_string()).join(&self.name)
    }

    #[must_use]
    pub const fn config(&self) -> Option<&AddonConfig> {
        self.config.as_ref()
    }

    #[must_use]
    pub const fn build_data(&self) -> &BuildData {
        &self.build_data
    }

    /// Scan for addons in both locations
    ///
    /// # Errors
    /// - [`Error::AddonLocationInvalid`] if a location is invalid
    /// - [`Error::AddonLocationInvalid`] if a folder name is invalid
    pub fn scan(root: &Path) -> Result<Vec<Self>, crate::error::Error> {
        let mut addons = Vec::new();
        for location in [Location::Addons, Location::Optionals] {
            if let Some(scanned) = location.scan(root)? {
                addons.extend(scanned);
            }
        }
        for addon in &addons {
            // I thought about creating a setting for this, but I don't want
            // it to end up in some HEMTT template that everyone just copies
            // and becomes irrelevant. I don't like this solutution either,
            // but I dislike it slightly less.
            if addon.name().to_lowercase() != addon.name() && !addon.name().starts_with("CUP_") {
                warn!(
                    "Addon name {} is not lowercase, it is highly recommended to use lowercase names",
                    addon.name()
                );
            }
            if addons.iter().any(|a| {
                a.name().to_lowercase() == addon.name().to_lowercase() && a.name() != addon.name()
            }) {
                return Err(crate::error::Error::Addon(Error::NameDuplicate(
                    addon.name().to_string(),
                )));
            }
            if addons.iter().any(|a| {
                a.name().to_lowercase() == addon.name().to_lowercase()
                    && a.location() != addon.location()
            }) {
                return Err(crate::error::Error::Addon(Error::Duplicate(
                    addon.name().to_string(),
                )));
            }
        }
        Ok(addons)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Location {
    Addons,
    Optionals,
}

impl Location {
    /// Scan for addons in this location
    ///
    /// # Errors
    /// - [`Error::AddonLocationInvalid`] if a folder name is invalid
    pub fn scan(self, root: &Path) -> Result<Option<Vec<Addon>>, crate::error::Error> {
        let folder = root.join(self.to_string());
        if !folder.exists() {
            return Ok(None);
        }
        trace!("Scanning {} for addons", folder.display());
        std::fs::read_dir(folder)?
            .collect::<std::io::Result<Vec<DirEntry>>>()?
            .iter()
            .map(std::fs::DirEntry::path)
            .filter(|file_or_dir| file_or_dir.is_dir())
            .map(|file| {
                let Some(name) = file.file_name() else {
                    return Err(crate::error::Error::Addon(Error::LocationInvalid(
                        file.display().to_string(),
                    )));
                };
                let Some(name) = name.to_str() else {
                    return Err(crate::error::Error::Addon(Error::LocationInvalid(
                        file.display().to_string(),
                    )));
                };
                Addon::new(root, name.to_string(), self)
            })
            .collect::<Result<Vec<Addon>, crate::error::Error>>()
            .map(Some)
    }
}

impl FromStr for Location {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "addons" => Ok(Self::Addons),
            "optionals" => Ok(Self::Optionals),
            _ => Err(Error::LocationInvalid(s.to_string())),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Addons => "addons",
                Self::Optionals => "optionals",
            }
        )
    }
}

type RequiredVersion = (Version, WorkspacePath, Range<usize>);
pub type UsedFunctions = Vec<(String, Position, Mapping, Mapping, WorkspacePath)>;
pub type DefinedFunctions = HashSet<(String, Arc<str>)>;
pub type MagazineWellInfo = (Vec<String>, Vec<(String, Arc<dyn Code>)>);

#[derive(Debug, Clone, Default)]
pub struct BuildData {
    required_version: Arc<RwLock<Option<RequiredVersion>>>,
    localizations: Arc<Mutex<Vec<(String, Position)>>>,
    functions_defined: Arc<Mutex<DefinedFunctions>>,
    functions_used: Arc<Mutex<UsedFunctions>>,
    magazine_well_info: Arc<Mutex<MagazineWellInfo>>,
}

impl BuildData {
    #[must_use]
    pub fn new() -> Self {
        Self {
            required_version: Arc::new(RwLock::new(None)),
            localizations: Arc::new(Mutex::new(Vec::new())),
            functions_defined: Arc::new(Mutex::new(HashSet::new())),
            functions_used: Arc::new(Mutex::new(Vec::new())),
            magazine_well_info: Arc::new(Mutex::new((Vec::new(), Vec::new()))),
        }
    }

    #[must_use]
    /// Fetches the required version
    ///
    /// Does not lock, the value is only accurate at the time of calling
    /// but it shouldn't change during normal HEMTT usage
    ///
    /// # Panics
    /// Panics if the lock is poisoned
    pub fn required_version(&self) -> Option<RequiredVersion> {
        self.required_version
            .read()
            .expect("the required version lock is poisoned")
            .clone()
    }

    /// Sets the required version
    ///
    /// # Panics
    /// Panics if the lock is poisoned
    pub fn set_required_version(&self, version: Version, file: WorkspacePath, line: Range<usize>) {
        *self
            .required_version
            .write()
            .expect("the required version lock is poisoned") = Some((version, file, line));
    }

    #[must_use]
    /// Fetches the localizations
    pub fn localizations(&self) -> Arc<Mutex<Vec<(String, Position)>>> {
        self.localizations.clone()
    }
    #[must_use]
    /// Fetches the used functions
    pub fn functions_used(&self) -> Arc<Mutex<UsedFunctions>> {
        self.functions_used.clone()
    }
    #[must_use]
    /// Fetches the defined functions
    pub fn functions_defined(&self) -> Arc<Mutex<DefinedFunctions>> {
        self.functions_defined.clone()
    }
    #[must_use]
    /// Fetches the `MagazineWellInfos` (tuple of missing mag and error)
    pub fn magazine_well_info(&self) -> Arc<Mutex<MagazineWellInfo>> {
        self.magazine_well_info.clone()
    }
}

mod test_helper {
    impl super::Addon {
        #[must_use]
        /// # Panics
        /// Panics if the prefix cannot be created
        pub fn test_addon() -> Self {
            Self {
                name: "test".to_string(),
                location: super::Location::Addons,
                config: None,
                prefix: super::Prefix::new("test").expect("test prefix"),
                build_data: super::BuildData::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn location_from_str() {
        assert_eq!("addons".parse(), Ok(super::Location::Addons));
        assert_eq!("optionals".parse(), Ok(super::Location::Optionals));
        assert_eq!(
            "foobar".parse::<super::Location>(),
            Err(super::Error::LocationInvalid("foobar".to_string()))
        );
    }
}
