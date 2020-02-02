use crate::{Addon, AddonLocation, HEMTTError};

fn addon_matches(name: &str, pattern: &str) -> bool {
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

pub fn get_from_args(args: &clap::ArgMatches) -> Result<Vec<Addon>, HEMTTError> {
    let all = args.value_of("addons").unwrap_or("") == "all";
    let mut addons: Vec<Addon> = if args.is_present("addons") && !all {
        get_from_location(&AddonLocation::Addons)?
            .into_iter()
            .filter(|a| args.values_of("addons").unwrap().any(|x| addon_matches(a.name.as_str(), x)))
            .collect()
    } else if all || (!args.is_present("opts") && !args.is_present("compats")) {
        get_from_location(&AddonLocation::Addons)?
    } else {
        Vec::new()
    };
    if args.is_present("opts") {
        addons.extend(if args.value_of("opts").unwrap_or("") == "all" {
            get_from_location(&AddonLocation::Optionals)?
        } else {
            get_from_location(&AddonLocation::Optionals)?
                .into_iter()
                .filter(|a| args.values_of("opts").unwrap().any(|x| addon_matches(a.name.as_str(), x)))
                .collect()
        });
    }
    if args.is_present("compats") {
        addons.extend(if args.value_of("compats").unwrap_or("") == "all" {
            get_from_location(&AddonLocation::Compats)?
        } else {
            get_from_location(&AddonLocation::Compats)?
                .into_iter()
                .filter(|a| args.values_of("compats").unwrap().any(|x| addon_matches(a.name.as_str(), x)))
                .collect()
        });
    }
    if !args.is_present("addons") && !args.is_present("opts") && !args.is_present("compats") {
        addons.extend(get_from_locations(&[AddonLocation::Optionals, AddonLocation::Compats])?);
    }
    Ok(addons)
}

pub fn get_all() -> Result<Vec<Addon>, HEMTTError> {
    get_from_locations(&AddonLocation::all())
}

pub fn get_from_locations(locations: &[AddonLocation]) -> Result<Vec<Addon>, HEMTTError> {
    let mut addons = Vec::new();
    for location in locations {
        if location.exists() {
            addons.extend(get_from_location(location)?);
        }
    }
    Ok(addons)
}

pub fn get_from_location(location: &AddonLocation) -> Result<Vec<Addon>, HEMTTError> {
    Ok(std::fs::read_dir(location.to_string())?
        .map(|file| file.unwrap().path())
        .filter(|file_or_dir| file_or_dir.is_dir())
        .map(|file| Addon {
            name: file.file_name().unwrap().to_str().unwrap().to_owned(),
            location: location.clone(),
        })
        .collect())
}
