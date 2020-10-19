mod defaults;
mod project;

pub use project::Project;

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
    Ok(std::fs::read_dir(location.to_string())?
        .map(|file| file.unwrap().path())
        .filter(|file_or_dir| file_or_dir.is_dir())
        .map(|file| Addon {
            name: file.file_name().unwrap().to_str().unwrap().to_owned(),
            location: *location,
        })
        .collect())
}
