use crate::{Error, config::global::launch::LaunchConfig};

mod launch;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Global configuration for HEMTT
pub struct GlobalConfig {
    launch: LaunchConfig,
}

impl GlobalConfig {
    #[must_use]
    /// Launch configuration
    pub const fn launch(&self) -> &LaunchConfig {
        &self.launch
    }

    /// Read a global config file from disk
    ///
    /// # Errors
    /// [`crate::error::Error::Io`] if the file cannot be read
    /// [`crate::error::Error::Toml`] if the file is not valid toml
    /// [`crate::error::Error::Prefix`] if the prefix is invalid
    pub fn from_file(path: &std::path::Path) -> Result<Self, Error> {
        Ok(GlobalConfigFile::from_file(path)?.into())
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfigFile::default().into()
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, serde::Deserialize)]
/// Global configuration file
pub struct GlobalConfigFile {
    #[serde(default)]
    launch: launch::LaunchConfigFile,
}

impl From<GlobalConfigFile> for GlobalConfig {
    fn from(file: GlobalConfigFile) -> Self {
        Self {
            launch: file.launch.into(),
        }
    }
}

impl GlobalConfigFile {
    pub fn from_file(path: &std::path::Path) -> Result<Self, crate::error::Error> {
        let content = fs_err::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}
