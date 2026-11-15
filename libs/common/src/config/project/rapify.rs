use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RapifyConfig {
    add_units: AutoAdd,
    add_weapons: AutoAdd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoAdd {
    Bool(bool),
    Prefixes(Vec<String>),
}

impl RapifyConfig {
    #[must_use]
    /// Whether to automatically handle units
    pub const fn add_units_enabled(&self) -> bool {
        match &self.add_units {
            AutoAdd::Bool(b) => *b,
            AutoAdd::Prefixes(_) => true,
        }
    }

    pub const fn auto_units(&self) -> &AutoAdd {
        &self.add_units
    }

    #[must_use]
    /// Whether to automatically handle weapons
    pub const fn add_weapons_enabled(&self) -> bool {
        match &self.add_weapons {
            AutoAdd::Bool(b) => *b,
            AutoAdd::Prefixes(_) => true,
        }
    }

    pub const fn auto_weapons(&self) -> &AutoAdd {
        &self.add_weapons
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct RapifySectionFile {
    #[serde(default)]
    add_units: AutoAddFile,
    #[serde(default)]
    add_weapons: AutoAddFile,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AutoAddFile {
    Bool(bool),
    Prefixes(Vec<String>),
    String(String),
}

impl Default for AutoAddFile {
    fn default() -> Self {
        Self::Bool(true)
    }
}

impl From<AutoAddFile> for AutoAdd {
    fn from(value: AutoAddFile) -> Self {
        match value {
            AutoAddFile::Bool(b) => Self::Bool(b),
            AutoAddFile::Prefixes(p) => Self::Prefixes(p),
            AutoAddFile::String(s) => Self::Prefixes(vec![s]),
        }
    }
}

impl From<RapifySectionFile> for RapifyConfig {
    fn from(file: RapifySectionFile) -> Self {
        Self {
            add_units: file.add_units.into(),
            add_weapons: file.add_weapons.into(),
        }
    }
}
