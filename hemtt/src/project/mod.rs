use config::{Config, Environment, File};
use std::collections::HashMap;
use std::path::PathBuf;
use vfs::PhysicalFS;

use semver::Version;
use serde::{Deserialize, Serialize};

mod defaults;
use defaults::*;

use crate::{Addon, AddonLocation, HEMTTError};

pub fn addon_matches(name: &str, pattern: &str) -> bool {
    let name = name.to_lowercase();
    let pattern = pattern.to_lowercase();
    if name == pattern {
        return true;
    }
    if let Ok(pat) = glob::Pattern::new(&pattern) {
        return pat.matches(&name);
    }
    false
}

pub fn get_all_addons() -> Result<Vec<Addon>, HEMTTError> {
    get_addon_from_locations(&AddonLocation::first_class())
}

pub fn get_addon_from_locations(locations: &[AddonLocation]) -> Result<Vec<Addon>, HEMTTError> {
    let mut addons = Vec::new();
    for location in locations {
        if location.exists() {
            addons.extend(get_addon_from_location(location)?);
        }
    }
    Ok(addons)
}

pub fn get_addon_from_location(location: &AddonLocation) -> Result<Vec<Addon>, HEMTTError> {
    std::fs::read_dir(location.to_string())?
        .map(|file| file.unwrap().path())
        .filter(|file_or_dir| file_or_dir.is_dir())
        .map(|file| {
            Addon::new(
                file.file_name().unwrap().to_str().unwrap().to_owned(),
                *location,
            )
        })
        .collect()
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Project {
    name: String,
    prefix: String,
    author: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default = "String::new")]
    template: String,

    #[serde(default = "default_version")]
    version: Version,

    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default = "String::new")]
    pub modname: String,

    #[serde(default = "default_mainprefix")]
    pub mainprefix: String,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default = "HashMap::new")]
    #[serde(rename(deserialize = "headerexts"))] // DEPRECATED
    #[serde(rename(deserialize = "header_exts"))]
    pub header_exts: HashMap<String, String>,

    // Files
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "default_include")]
    pub include: Vec<PathBuf>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub exclude: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub files: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default = "default_folder_optionals")]
    pub folder_optionals: Option<bool>,

    // Signing
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default = "default_reuse_private_key")]
    pub reuse_private_key: Option<bool>,

    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default = "String::new")]
    #[serde(rename(deserialize = "keyname"))] // DEPRECATED
    #[serde(rename(deserialize = "key_name"))]
    key_name: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default = "String::new")]
    #[serde(rename(deserialize = "signame"))] // DEPRECATED
    #[serde(rename(deserialize = "sig_name"))] // DEPRECATED
    #[serde(rename(deserialize = "authority"))]
    pub authority: String,

    #[serde(default = "default_sig_version")]
    #[serde(rename(deserialize = "sigversion"))] // DEPRECATED
    #[serde(rename(deserialize = "sig_version"))]
    pub sig_version: u8,

    // Scripts
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub check: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub prebuild: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub postbuild: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub releasebuild: Vec<String>,
    // #[serde(skip_serializing_if = "HashMap::is_empty")]
    // #[serde(default = "HashMap::new")]
    // pub scripts: HashMap<String, crate::BuildScript>,
}

impl Project {
    /// Finds the root of the project
    pub fn find_root() -> Result<PathBuf, HEMTTError> {
        let mut dir = std::env::current_dir().unwrap();
        loop {
            let mut search = dir.clone();
            search.push(".hemtt");
            if search.exists() {
                search.pop();
                return Ok(search);
            } else {
                let mut search = dir.clone();
                search.push("hemtt.toml");
                if search.exists() {
                    search.pop();
                    return Ok(search);
                }
            }
            dir.pop();
            search.pop();
            if dir == search {
                return Err(HEMTTError::NoProjectFound);
            }
        }
    }

    pub fn fs() -> Result<PhysicalFS, HEMTTError> {
        Ok(PhysicalFS::new(Self::find_root()?))
    }

    pub fn new(name: String, prefix: String, author: String, template: String) -> Self {
        Self {
            name,
            prefix,
            author,
            template,

            version: default_version(),

            modname: String::new(),
            mainprefix: default_mainprefix(),

            header_exts: HashMap::new(),

            include: default_include(),
            exclude: Vec::new(),
            files: if std::path::Path::new("mod.cpp").exists() {
                vec!["mod.cpp".to_owned()]
            } else {
                Vec::new()
            },
            folder_optionals: default_folder_optionals(),

            reuse_private_key: default_reuse_private_key(),
            key_name: String::new(),
            authority: String::new(),
            sig_version: default_sig_version(),

            check: Vec::new(),
            postbuild: Vec::new(),
            prebuild: Vec::new(),
            releasebuild: Vec::new(),
            // scripts: HashMap::new(),
        }
    }

    pub fn read() -> Result<Self, HEMTTError> {
        let mut p = Config::new();
        let root = Self::find_root()?;
        debug!("Root Directory: {:?}", root);
        std::env::set_current_dir(root)?;

        if PathBuf::from("hemtt.toml").exists() {
            // Single file (toml)
            p.merge(File::with_name("hemtt.toml").required(true))
                .map_err(|e| HEMTTError::Generic(e.to_string()))?;
        } else {
            // Project folder
            if !PathBuf::from(".hemtt/").exists() {
                return Err(HEMTTError::NoProjectFound);
            }
            p.merge(File::with_name(".hemtt/base").required(true))
                .map_err(|e| HEMTTError::Generic(e.to_string()))?;
            // p.merge(File::with_name(&format!(".hemtt/{}", "base")).required(false)).map_err(|e| HEMTTError::Generic(e.to_string()));
            p.merge(File::with_name(".hemtt/local").required(false))
                .map_err(|e| HEMTTError::Generic(e.to_string()))?;
        }

        p.merge(Environment::with_prefix("app"))
            .map_err(|e| HEMTTError::Generic(e.to_string()))?;

        p.try_into().map_err(|e| HEMTTError::Generic(e.to_string()))
    }

    /// The name of the project
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The prefix used by addons
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// The author of the project
    pub fn author(&self) -> &str {
        &self.author
    }

    /// The project template, used for file generation
    pub fn template(&self) -> &str {
        &self.template
    }

    /// The version of the project
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Mutable version of the project
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.version
    }
}

impl From<&Project> for hemtt_handlebars::Variables {
    fn from(project: &Project) -> Self {
        use serde_json::{Map, Value};
        use std::collections::BTreeMap;
        Self({
            let mut map = BTreeMap::new();
            map.insert(
                String::from("project"),
                Value::Object({
                    let mut map = Map::new();
                    map.insert(String::from("name"), Value::String(project.name.clone()));
                    map.insert(
                        String::from("author"),
                        Value::String(project.author.clone()),
                    );
                    map.insert(
                        String::from("mainprefix"),
                        Value::String(project.mainprefix.clone()),
                    );
                    map.insert(
                        String::from("prefix"),
                        Value::String(project.prefix.clone()),
                    );
                    map.insert(
                        String::from("modname"),
                        Value::String(project.modname.clone()),
                    );
                    map
                }),
            );
            map
        })
    }
}
