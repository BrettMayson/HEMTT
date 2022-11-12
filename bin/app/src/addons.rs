use std::{fs::DirEntry, path::PathBuf, str::FromStr};

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Addon {
    name: String,
    location: Location,
}

impl Addon {
    pub const fn new(name: String, location: Location) -> Self {
        Self { name, location }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn location(&self) -> &Location {
        &self.location
    }

    pub fn folder(&self) -> String {
        format!("{}/{}", self.location.to_string(), self.name)
    }

    pub fn scan(locations: &[Location]) -> Result<Vec<Self>, Error> {
        let mut addons = Vec::new();
        for location in locations {
            addons.extend(location.scan()?);
        }
        Ok(addons)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    Addons,
    Compats,
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
            "compats" => Ok(Self::Compats),
            "optionals" => Ok(Self::Optionals),
            _ => Err(Error::InvalidAddonLocation(s.to_string())),
        }
    }
}

impl ToString for Location {
    fn to_string(&self) -> String {
        match self {
            Self::Addons => "addons",
            Self::Compats => "compats",
            Self::Optionals => "optionals",
        }
        .to_string()
    }
}
