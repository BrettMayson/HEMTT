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
    preset: Preset,

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
