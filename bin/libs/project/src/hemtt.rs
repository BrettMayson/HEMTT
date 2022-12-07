use hemtt_pbo::BISignVersion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Features {
    #[serde(default)]
    config: hemtt_config::Options,

    #[serde(default)]
    dev: DevOptions,

    #[serde(default)]
    build: BuildOptions,

    #[serde(default)]
    signing: SigningOptions,

    #[serde(default)]
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
    pub const fn dev(&self) -> &DevOptions {
        &self.dev
    }

    #[must_use]
    pub const fn build(&self) -> &BuildOptions {
        &self.build
    }

    #[must_use]
    pub const fn signing(&self) -> &SigningOptions {
        &self.signing
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
pub struct DevOptions {
    #[serde(default)]
    exclude: Vec<String>,
}

impl DevOptions {
    #[must_use]
    pub fn exclude(&self) -> &[String] {
        &self.exclude
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BuildOptions {
    #[serde(default)]
    /// Should optionals be built into their own mod?
    /// Default: true
    optional_mod_folders: Option<bool>,
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
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SigningOptions {
    version: Option<BISignVersion>,

    #[serde(default)]
    authority: Option<String>,

    #[serde(default)]
    include_git_hash: Option<bool>,
}

impl SigningOptions {
    #[must_use]
    pub const fn version(&self) -> BISignVersion {
        if let Some(version) = self.version {
            version
        } else {
            BISignVersion::V3
        }
    }

    /// Get the authority for signing
    ///
    /// # Errors
    /// Returns an error if the authority is not set
    pub fn authority(&self) -> Result<String, hemtt_signing::Error> {
        self.authority.as_ref().map_or_else(
            || Err(hemtt_signing::Error::MissingAuthority),
            |authority| Ok(authority.clone()),
        )
    }

    #[must_use]
    pub const fn include_git_hash(&self) -> bool {
        if let Some(include) = self.include_git_hash {
            include
        } else {
            false
        }
    }
}
