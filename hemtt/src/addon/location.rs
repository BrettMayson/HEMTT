use std::path::PathBuf;

use strum_macros::EnumIter;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, EnumIter, Hash)]
pub enum AddonLocation {
    Addons,
    Compats,
    Optionals,
    // Custom(String),
}

impl AddonLocation {
    /// The addon location exists on disk
    pub fn exists(&self) -> bool {
        PathBuf::from(self).exists()
    }

    /// Is the location a supported location
    pub fn is_first_class(&self) -> bool {
        match *self {
            Self::Addons => true,
            Self::Compats => true,
            Self::Optionals => true,
            // _ => false,
        }
    }

    /// CLI - Is the location a valid target
    pub fn validate(location: String) -> Result<(), String> {
        // Currently only first class locations are supported
        if AddonLocation::from(location.as_str()).is_first_class() {
            Ok(())
        } else {
            Err(Self::options())
        }
    }

    /// CLI - Valid options for CLI interfaces
    pub fn options() -> String {
        format!(
            "options are: {}",
            Self::first_class()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }

    /// List of first class locations
    pub fn first_class() -> Vec<Self> {
        vec![Self::Addons, Self::Compats, Self::Optionals]
    }
}

impl std::fmt::Display for AddonLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Addons => write!(f, "addons"),
            Self::Compats => write!(f, "compats"),
            Self::Optionals => write!(f, "optionals"),
            // Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl std::fmt::Debug for AddonLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Addons => String::from("standard(addons)"),
                Self::Compats => String::from("standard(compats)"),
                Self::Optionals => String::from("standard(optionals)"),
                // Self::Custom(s) => format!("custom({})", s),
            }
        )
    }
}

impl From<&str> for AddonLocation {
    fn from(loc: &str) -> Self {
        match loc.to_lowercase().as_str() {
            "addons" => Self::Addons,
            "compats" => Self::Compats,
            "optionals" => Self::Optionals,
            // TODO bring back custom
            _ => panic!("Invalid AddonLocation"), // _ => Self::Custom(loc.to_owned()),
        }
    }
}

impl From<AddonLocation> for PathBuf {
    fn from(al: AddonLocation) -> Self {
        PathBuf::from(al.to_string())
    }
}

impl From<&AddonLocation> for PathBuf {
    fn from(al: &AddonLocation) -> Self {
        PathBuf::from(al.to_string())
    }
}
