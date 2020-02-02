use std::collections::BTreeMap;
use std::path::PathBuf;

use handlebars::to_json;
use serde_json::value::Value as Json;

use crate::{HEMTTError, Project};

mod location;
pub use location::AddonLocation;

#[derive(Debug)]
pub struct Addon {
    pub name: String,
    pub location: AddonLocation,
}
impl Addon {
    pub fn folder(&self) -> PathBuf {
        PathBuf::from(format!(
            "{}{}{}",
            self.location.to_string(),
            std::path::MAIN_SEPARATOR,
            self.name
        ))
    }

    pub fn target(&self, p: &Project) -> PathBuf {
        let mut target = PathBuf::from(self.location.to_string());
        if !p.prefix.is_empty() {
            target.push(&format!("{}_{}.pbo", p.prefix, self.name));
        } else {
            target.push(&format!("{}.pbo", self.name));
        }
        target
    }

    pub fn get_variables(&self, p: &Project) -> BTreeMap<&'static str, Json> {
        let mut vars = p.get_variables();
        vars.insert("folder", to_json(self.folder()));
        vars.insert("addon", to_json(self.name.clone()));
        vars.insert("target", to_json(self.target(p).to_str().to_owned()));
        vars
    }

    /// Folder containing the released addon
    pub fn release_location(&self, release_folder: &PathBuf, p: &Project) -> PathBuf {
        let mut r = release_folder.clone();
        r.push(self.location.to_string());

        // Folder / Launchable Optionals
        if p.folder_optionals() && (self.location == AddonLocation::Compats || self.location == AddonLocation::Optionals) {
            r.push(&format!("@{}_{}", p.modname().unwrap(), self.name));
            r.push("addons");
        }

        r
    }

    /// File path of the released addon
    pub fn release_target(&self, release_folder: &PathBuf, p: &Project) -> PathBuf {
        let mut r = self.release_location(release_folder, p);

        if !p.prefix.is_empty() {
            r.push(&format!("{}_{}.pbo", p.prefix, self.name));
        } else {
            r.push(&format!("{}.pbo", self.name));
        }
        r
    }

    /// Moves the released pbo to the `release_target`
    pub fn release(&self, release_folder: &PathBuf, p: &Project) -> Result<(), HEMTTError> {
        let target = self.release_target(release_folder, p);
        create_dir!(target.parent().unwrap())?;
        copy_file!(self.target(&p), target)?;
        Ok(())
    }
}
