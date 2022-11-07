use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Features {
    config: hemtt_config::Options,

    pbo_prefix_allow_leading_slash: Option<bool>,
}

impl Features {
    #[must_use]
    pub const fn config(&self) -> &hemtt_config::Options {
        &self.config
    }

    #[must_use]
    pub const fn pbo_prefix_allow_leading_slash(&self) -> bool {
        if let Some(allow) = self.pbo_prefix_allow_leading_slash {
            allow
        } else {
            false
        }
    }
}
