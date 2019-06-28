use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::value::{Value as Json};
use handlebars::to_json;

pub mod build;
pub mod checks;
pub mod prebuild;

use crate::{HEMTTError, Project};

#[derive(Debug, Clone)]
pub enum AddonLocation {
    Addons,
    Optionals,
    Compats,
}

#[derive(Debug)]
pub struct Addon {
    pub name: String,
    pub location: AddonLocation,
}
impl Addon {
    pub fn folder(&self) -> PathBuf {
        PathBuf::from(format!("{}{}{}", folder_name(&self.location), std::path::MAIN_SEPARATOR, self.name))
    }

    pub fn target(&self, p: &Project) -> PathBuf {
        let mut target = PathBuf::from(crate::build::folder_name(&self.location));
        target.push(&format!("{}_{}.pbo", p.prefix, self.name));
        target
    }

    pub fn get_variables(&self, p: &Project) -> BTreeMap<&'static str, Json> {
        let mut vars = p.get_variables();
        vars.insert("folder", to_json(self.folder()));
        vars.insert("addon", to_json(self.name.clone()));
        vars
    }
}

pub fn folder_name(location: &AddonLocation) -> String {
    String::from(match location {
        AddonLocation::Addons => "addons",
        AddonLocation::Optionals => "optionals",
        AddonLocation::Compats => "compats",
    })
}

pub fn get_addons(location: AddonLocation) -> Result<Vec<Addon>, HEMTTError> {
    Ok(std::fs::read_dir(folder_name(&location))?
        .map(|file| file.unwrap().path())
        .filter(|file_or_dir| file_or_dir.is_dir())
        .map(|file| Addon {
            name: file.file_name().unwrap().to_str().unwrap().to_owned(),
            location: location.clone(),
        })
        .collect())
}
