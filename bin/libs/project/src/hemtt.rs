use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Features {
    config: hemtt_config::Options,
    build: BuildOptions,

    /// Can PBO prefixes have a leading slash?
    ///
    /// Default: false
    pbo_prefix_allow_leading_slash: Option<bool>,
}

impl Features {
    #[must_use]
    pub const fn config(&self) -> &hemtt_config::Options {
        &self.config
    }

    #[must_use]
    pub const fn build(&self) -> &BuildOptions {
        &self.build
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BuildOptions {
    #[serde(default)]
    /// Should optionals be built into their own mod?
    /// Default: true
    optional_mod_folders: Option<bool>,
    /// Should compats be built into their own mod?
    /// Default: true
    #[serde(default)]
    compat_mod_folders: Option<bool>,
}

impl BuildOptions {
    #[must_use]
    pub const fn optional_mod_folders(&self) -> bool {
        if let Some(optional) = self.optional_mod_folders {
            optional
        } else {
            true
        }
    }

    #[must_use]
    pub const fn compat_mod_folders(&self) -> bool {
        if let Some(compat) = self.compat_mod_folders {
            compat
        } else {
            true
        }
    }
}
