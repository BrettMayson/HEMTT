use std::{fs::DirEntry, path::PathBuf, str::FromStr};

use hemtt_bin_config::addon::Configuration;
use hemtt_bin_error::Error;

#[derive(Debug, Clone)]
pub struct Addon {
    name: String,
    location: Location,
    config: Option<Configuration>,
}

impl Addon {
    pub fn new(name: String, location: Location) -> Self {
        Self {
            config: {
                let path =
                    PathBuf::from(format!("{}/{}", location.to_string(), name)).join("addon.toml");
                if path.exists() {
                    Some(Configuration::from_file(&path).unwrap())
                } else {
                    None
                }
            },
            location,
            name,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pbo_name(&self, prefix: &str) -> String {
        format!("{prefix}_{}", self.name)
    }

    pub const fn location(&self) -> &Location {
        &self.location
    }

    pub fn folder(&self) -> String {
        format!("{}/{}", self.location.to_string(), self.name)
    }

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
        // TODO scope to root
        Ok(std::fs::read_dir(self.to_string())?
            .collect::<std::io::Result<Vec<DirEntry>>>()?
            .iter()
            .map(std::fs::DirEntry::path)
            .filter(|file_or_dir| file_or_dir.is_dir())
            .map(|file| Addon::new(file.file_name().unwrap().to_str().unwrap().to_owned(), self))
            .collect())
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
