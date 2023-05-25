#![allow(clippy::use_self)] // serde false positive

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Preset of options to use for parsing
///
/// HEMTT: Superset of BI with some QOL improvements
/// BI: Match a strict definition of BI's parser
pub enum Preset {
    /// Superset of BI with some QOL improvements
    Hemtt,
    #[default]
    /// Match a strict definition of BI's parser
    Bi,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
/// Options for parsing
pub struct Options {
    #[serde(default = "default_preset")]
    /// Preset to use for config parsing
    ///
    /// BI: Match a strict definition of BI's parser
    /// HEMTT: Superset of BI with some QOL improvements
    pub(crate) preset: Preset,

    /// Can arrays have trailing commas?
    ///
    /// See [`Options::array_allow_trailing_comma`]
    pub(crate) array_allow_trailing_comma: Option<bool>,
}

impl Options {
    #[must_use]
    /// Create a new set of options from a preset
    pub fn from_preset(preset: Preset) -> Self {
        Self {
            preset,
            ..Default::default()
        }
    }

    #[must_use]
    /// Can arrays have trailing commas?
    ///
    /// Default (BI): `false`
    /// Default (HEMTT): `true`
    ///
    /// When false, the following is invalid:
    /// ```cpp
    /// my_array[] = {
    ///     1,
    ///     2, // <- trailing comma on last element
    /// };
    /// ```
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
    Preset::Bi
}
