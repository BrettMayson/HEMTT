use std::path::PathBuf;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AddonLocation {
    Addons,
    Compats,
    Optionals,
    Custom(String),
}

impl AddonLocation {
    pub fn exists(&self) -> bool {
        PathBuf::from(self).exists()
    }

    pub fn first_class() -> Vec<Self> {
        vec![Self::Addons, Self::Compats, Self::Optionals]
    }
}

impl ToString for AddonLocation {
    fn to_string(&self) -> String {
        String::from(match self {
            Self::Addons => "addons",
            Self::Compats => "compats",
            Self::Optionals => "optionals",
            Self::Custom(s) => s,
        })
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
