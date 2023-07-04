use std::{fs::DirEntry, path::PathBuf, str::FromStr};

use hemtt_pbo::{prefix, Prefix};

use crate::{config::addon::Configuration, error::Error};

#[derive(Debug, Clone)]
pub struct Addon {
    name: String,
    location: Location,
    config: Option<Configuration>,
    prefix: Prefix,
}

impl Addon {
    pub fn new(name: String, location: Location) -> Result<Self, Error> {
        let path = PathBuf::from(location.to_string()).join(&name);
        Ok(Self {
            config: {
                let path = path.join("addon.toml");
                if path.exists() {
                    Some(Configuration::from_file(&path).unwrap())
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
                        let content = std::fs::read_to_string(path).unwrap();
                        prefix = Some(Prefix::new(&content).unwrap());
                        break 'search;
                    }
                }
                if prefix.is_none() {
                    return Err(Error::AddonPrefixMissing(name));
                }
                prefix.unwrap()
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
    pub const fn config(&self) -> Option<&Configuration> {
        self.config.as_ref()
    }

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
    pub fn scan(self) -> Result<Vec<Addon>, Error> {
        if !PathBuf::from(self.to_string()).exists() {
            return Ok(Vec::new());
        }
        std::fs::read_dir(self.to_string())?
            .collect::<std::io::Result<Vec<DirEntry>>>()?
            .iter()
            .map(std::fs::DirEntry::path)
            .filter(|file_or_dir| file_or_dir.is_dir())
            .map(|file| Addon::new(file.file_name().unwrap().to_str().unwrap().to_owned(), self))
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
