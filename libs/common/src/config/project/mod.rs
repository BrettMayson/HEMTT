//! Module for reading HEMTT project files

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Once, RwLock},
};

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::Error;

use super::deprecated;

pub mod files;
pub mod hemtt;
pub mod lint;
pub mod signing;
pub mod version;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Configuration for a HEMTT project
pub struct ProjectConfig {
    /// The name of the project
    name: String,
    /// Prefix for the project
    prefix: String,
    /// Main prefix for the project
    mainprefix: Option<String>,

    /// version of the project
    version: version::VersionConfig,

    /// Properties to be added to built PBOs
    properties: HashMap<String, String>,

    /// Files to be included in the root of the project, supports glob patterns
    files: files::FilesConfig,

    /// Configuration for lints
    lints: lint::LintGroupConfig,

    /// HEMTT specific configuration
    hemtt: hemtt::HemttConfig,

    /// Signing specific configuration
    signing: signing::SigningConfig,
}

impl ProjectConfig {
    #[must_use]
    /// The name of the project
    pub const fn name(&self) -> &String {
        &self.name
    }

    #[must_use]
    /// Prefix for the project
    pub const fn prefix(&self) -> &String {
        &self.prefix
    }

    #[must_use]
    /// Main prefix for the project
    pub const fn mainprefix(&self) -> Option<&String> {
        self.mainprefix.as_ref()
    }

    #[must_use]
    /// version of the project
    pub const fn version(&self) -> &version::VersionConfig {
        &self.version
    }

    #[must_use]
    /// Properties to be added to built PBOs
    pub const fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    #[must_use]
    /// Files to be included in the root of the project, supports glob patterns
    pub const fn files(&self) -> &files::FilesConfig {
        &self.files
    }

    #[must_use]
    /// Configuration for lints
    pub const fn lints(&self) -> &lint::LintGroupConfig {
        &self.lints
    }

    #[must_use]
    /// HEMTT specific configuration
    pub const fn hemtt(&self) -> &hemtt::HemttConfig {
        &self.hemtt
    }

    #[must_use]
    /// Signing specific configuration
    pub const fn signing(&self) -> &signing::SigningConfig {
        &self.signing
    }

    /// Read a project file from disk
    ///
    /// # Errors
    /// [`crate::error::Error::Io`] if the file cannot be read
    /// [`crate::error::Error::Toml`] if the file is not valid toml
    /// [`crate::error::Error::Prefix`] if the prefix is invalid
    pub fn from_file(path: &std::path::Path) -> Result<Self, Error> {
        ProjectFile::from_file(path)?.try_into()
    }
}

#[derive(Debug, Clone)]
/// Information about the build
pub struct BuildInfo {
    /// lowercase, e.g. `$str_prefix_`
    config_string_prefix: String,
    /// lowercase, e.g. `str_prefix_`
    code_string_prefix: String,
    /// All defined stringtable entries in lowercase, modified during the build
    stringtable: Arc<RwLock<HashSet<String>>>,
    /// using `hemtt release`
    is_release: bool,
    /// using `hemtt check --pedantic`
    is_pedantic: bool,
}

impl BuildInfo {
    #[must_use]
    pub fn new(prefix: &String) -> Self {
        Self {
            config_string_prefix: format!("$str_{prefix}_").to_lowercase(),
            code_string_prefix: format!("str_{prefix}_").to_lowercase(),
            stringtable: Arc::new(RwLock::new(HashSet::new())),
            is_release: false,
            is_pedantic: false,
        }
    }

    #[must_use]
    pub const fn is_release(&self) -> bool {
        self.is_release
    }
    #[must_use]
    pub const fn with_release(mut self, is_release: bool) -> Self {
        self.is_release = is_release;
        self
    }
    #[must_use]
    pub const fn is_pedantic(&self) -> bool {
        self.is_pedantic
    }
    #[must_use]
    pub const fn with_pedantic(mut self, is_pedantic: bool) -> Self {
        self.is_pedantic = is_pedantic;
        self
    }
    #[must_use]
    pub const fn stringtable_prefix(&self) -> &String {
        &self.code_string_prefix
    }
    #[must_use]
    pub fn stringtable_matches_project(&self, str: &str, is_config: bool) -> bool {
        if is_config {
            str.to_lowercase().starts_with(&self.config_string_prefix)
        } else {
            str.to_lowercase().starts_with(&self.code_string_prefix)
        }
    }
    #[must_use]
    /// # Panics
    pub fn stringtable_append(&self, str: &str) -> bool {
        self.stringtable
            .write()
            .expect("mutex saftey")
            .insert(str.to_lowercase())
    }
    #[must_use]
    /// # Panics
    pub fn stringtable_exists(&self, str: &str, is_config: bool) -> bool {
        let target = if is_config {
            (str[1..]).to_lowercase()
        } else {
            str.to_lowercase()
        };
        self.stringtable
            .read()
            .expect("mutex saftey")
            .contains(&target)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
/// Configuration for a project
pub struct ProjectFile {
    /// The name of the project
    name: String,
    /// Prefix for the project
    prefix: String,
    /// Main prefix for the project
    mainprefix: Option<String>,

    #[serde(default)]
    /// version of the project
    version: version::VersionSectionFile,

    #[serde(default)]
    /// Properties to be added to built PBOs
    properties: HashMap<String, String>,

    #[serde(default)]
    /// Files to be included in the root of the project, supports glob patterns
    files: files::FilesSectionFile,

    #[serde(default)]
    /// Lint configuration
    lints: lint::LintSectionFile,

    #[serde(default)]
    hemtt: hemtt::HemttSectionFile,

    #[serde(default)]
    signing: signing::SigningSectionFile,

    #[serde(skip)]
    meta_path: PathBuf,
}

static DEPRECATION: Once = Once::new();

impl ProjectFile {
    pub fn from_file(path: &std::path::Path) -> Result<Self, Error> {
        Self::from_str(&std::fs::read_to_string(path)?, &path.display().to_string())
    }

    pub fn from_str(content: &str, path: &str) -> Result<Self, Error> {
        DEPRECATION.call_once(|| {
            if content.contains("[hemtt.launch]") {
                deprecated(path, "[hemtt.launch]", "[hemtt.launch.default]", None);
            }

            if content.contains("[asc]") {
                warn!("ASC config is no longer used");
            }

            if content.contains("[lint]") {
                warn!("lint config is no longer used");
            }
        });

        let mut config: Self =
            toml::from_str(&content.replace("[hemtt.launch]", "[hemtt.launch.default]"))?;

        config.meta_path = PathBuf::from(path);

        Ok(config)
    }
}

impl TryFrom<ProjectFile> for ProjectConfig {
    type Error = Error;

    fn try_from(file: ProjectFile) -> Result<Self, Self::Error> {
        if file.prefix.is_empty() {
            return Err(Error::Prefix(crate::prefix::Error::Empty));
        }

        let ret = Self {
            hemtt: file.hemtt.into_config(&file.meta_path, &file.prefix)?,
            name: file.name,
            prefix: file.prefix,
            mainprefix: file.mainprefix,
            version: file.version.try_into()?,
            properties: file.properties,
            files: file.files.into(),
            lints: file.lints.into(),
            signing: file.signing.into(),
        };

        let mut lints_path = file.meta_path;
        lints_path.set_file_name("lints.toml");
        let lints_source = if lints_path.exists() {
            if ret.lints.is_empty() {
                let lints_source = std::fs::read_to_string(&lints_path)?;
                if lints_source.contains("[lints.") {
                    return Err(Error::ConfigInvalid(
                        "Configs in `lints.toml` do not need to be under `[lints.*]`.".to_string(),
                    ));
                }
                toml::from_str::<lint::LintSectionFile>(&lints_source)?.into()
            } else {
                return Err(Error::LintsConfigConflict);
            }
        } else {
            ret.lints
        };

        Ok(Self {
            lints: lints_source,
            ..ret
        })
    }
}

mod test_helper {
    use std::collections::HashMap;

    use super::{files, hemtt, lint, signing, version};

    impl super::ProjectConfig {
        #[must_use]
        /// Create a test project configuration
        ///
        /// # Panics
        /// Panics if the test project cannot be converted
        pub fn test_project() -> Self {
            super::ProjectFile {
                name: "Advanced Banana Environment".to_string(),
                prefix: "abe".to_string(),
                mainprefix: None,
                version: version::VersionSectionFile::default(),
                properties: HashMap::default(),
                files: files::FilesSectionFile::default(),
                lints: lint::LintSectionFile::default(),
                hemtt: hemtt::HemttSectionFile::default(),
                signing: signing::SigningSectionFile::default(),
                meta_path: std::path::PathBuf::default(),
            }
            .try_into()
            .expect("Failed to convert test ProjectFile to ProjectConfig")
        }
    }

    #[test]
    fn test_test_project() {
        let _ = super::ProjectConfig::test_project();
    }
}
