use std::{collections::HashMap, path::Path, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{hemtt::Features, Error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    project: Project,
    #[serde(default)]
    hemtt: Features,
}

impl Configuration {
    #[must_use]
    pub const fn project(&self) -> &Project {
        &self.project
    }

    #[must_use]
    pub const fn hemtt(&self) -> &Features {
        &self.hemtt
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    name: String,
    #[serde(default)]
    headers: HashMap<String, String>,
    #[serde(default)]
    files: Vec<String>,
}

impl Project {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
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
        files
    }
}
