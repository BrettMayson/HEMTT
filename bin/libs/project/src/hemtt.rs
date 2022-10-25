use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum Preset {
    #[default]
    HEMTT,
    BI,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Features {
    #[serde(default = "default_preset")]
    preset: Preset,

    pbo_prefix_allow_leading_slash: Option<bool>,
}

impl Features {
    #[must_use]
    pub const fn pbo_prefix_allow_leading_slash(&self) -> bool {
        if let Some(allow) = self.pbo_prefix_allow_leading_slash {
            allow
        } else {
            match self.preset {
                Preset::HEMTT => true,
                Preset::BI => false,
            }
        }
    }
}

const fn default_preset() -> Preset {
    Preset::HEMTT
}
