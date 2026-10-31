use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Configuration for `hemtt release`
pub struct PaaOptions {
    enabled: bool,
    extensions: Vec<String>,
    allow_argb8: Argb8Option,
}

#[derive(PartialEq, Eq, Default, Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Argb8Option {
    Allow,
    #[default]
    Ask,
    Deny,
}

impl PaaOptions {
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    /// Get the list of image file extensions that should be converted to PAA format
    pub const fn extensions(&self) -> &Vec<String> {
        &self.extensions
    }

    /// Get the ARGB8 option
    pub const fn allow_argb8(&self) -> Argb8Option {
        self.allow_argb8
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Release specific configuration
pub struct PaaOptionsFile {
    #[serde(default)]
    enabled: Option<bool>,

    #[serde(default)]
    extensions: Vec<String>,

    #[serde(default)]
    allow_argb8: Argb8OptionFile,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argb8OptionFile {
    Bool(bool),
    Enum(Argb8Option),
}

impl Default for Argb8OptionFile {
    fn default() -> Self {
        Self::Enum(Argb8Option::default())
    }
}

impl PaaOptionsFile {
    pub fn into_config(self) -> PaaOptions {
        let allow_argb8 = match self.allow_argb8 {
            Argb8OptionFile::Enum(option) => option,
            Argb8OptionFile::Bool(b) => {
                if b {
                    Argb8Option::Allow
                } else {
                    Argb8Option::Deny
                }
            }
        };
        PaaOptions {
            extensions: {
                let mut exts = self.extensions;
                exts.push(String::from("png"));
                exts.push(String::from("tga"));
                exts.sort();
                exts.dedup();
                exts
            },
            allow_argb8,
            enabled: self.enabled.unwrap_or(false),
        }
    }
}
