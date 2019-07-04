use std::collections::BTreeMap;
use std::path::PathBuf;

use handlebars::to_json;
use serde_json::value::{Value as Json};

use strum_macros::EnumIter;

use crate::Project;

#[derive(Clone, Debug, EnumIter, PartialEq)]
pub enum AddonLocation {
    Addons,
    Compats,
    Optionals,
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
        let mut target = PathBuf::from(crate::build::addon::folder_name(&self.location));
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
        AddonLocation::Compats => "compats",
        AddonLocation::Optionals => "optionals",
    })
}
