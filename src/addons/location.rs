use std::path::PathBuf;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Debug, EnumIter, PartialEq)]
pub enum AddonLocation {
    Addons,
    Compats,
    Optionals,
}
impl ToString for AddonLocation {
    fn to_string(&self) -> String {
        String::from(match self {
            AddonLocation::Addons => "addons",
            AddonLocation::Compats => "compats",
            AddonLocation::Optionals => "optionals",
        })
    }
}
impl AddonLocation {
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(self.to_string())
    }

    pub fn exists(&self) -> bool {
        self.to_path_buf().exists()
    }

    pub fn all() -> Vec<AddonLocation> {
        AddonLocation::iter().collect()
    }
}
