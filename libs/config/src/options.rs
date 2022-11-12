#![allow(clippy::use_self)] // serde false positive

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum Preset {
    Hemtt,
    #[default]
    Bi,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    #[serde(default = "default_preset")]
    /// Preset to use for config parsing
    ///
    /// BI: Match a strict definition of BI's parser
    /// HEMTT: Superset of BI with some QOL improvements
    preset: Preset,

    /// Can arrays have trailing commas?
    ///
    /// Default (BI): false
    /// Default (HEMTT): true
    array_allow_trailing_comma: Option<bool>,
}

impl Options {
    #[must_use]
    pub fn from_preset(preset: Preset) -> Self {
        Self {
            preset,
            ..Default::default()
        }
    }

    #[must_use]
    pub const fn array_allow_trailing_comma(&self) -> bool {
        if let Some(allow) = self.array_allow_trailing_comma {
            allow
        } else {
            match self.preset {
                Preset::Hemtt => true,
                Preset::Bi => false,
            }
        }
    }
}

const fn default_preset() -> Preset {
    Preset::Hemtt
}
