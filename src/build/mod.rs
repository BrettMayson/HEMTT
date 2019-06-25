use std::collections::BTreeMap;

use serde_json::value::{Value as Json};
use handlebars::to_json;

pub mod checks;
pub mod prebuild;

use crate::HEMTTError;
use crate::project::Project;

#[derive(Debug, Clone)]
pub enum AddonLocation {
    Addons,
    Optionals,
    Compats,
}

pub struct Addon {
    pub name: String,
    pub location: AddonLocation,
}
impl Addon {
    pub fn folder(&self) -> String {
        format!("{}/{}", folder_name(&self.location), self.name)
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
        .map(|file| Addon {name: file.file_name().unwrap().to_str().unwrap().to_owned(), location: location.clone()})
        .collect())
}
