use std::collections::HashMap;

use codespan_reporting::diagnostic::Severity;
use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Lint group config
pub struct LintGroupConfig {
    /// Lints for config
    config: HashMap<String, LintConfigOverride>,
    sqf: HashMap<String, LintConfigOverride>,
}

impl LintGroupConfig {
    #[must_use]
    /// Get the lints
    pub const fn config(&self) -> &HashMap<String, LintConfigOverride> {
        &self.config
    }

    #[must_use]
    /// Get the sqf lints
    pub const fn sqf(&self) -> &HashMap<String, LintConfigOverride> {
        &self.sqf
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq)]
pub struct LintConfig {
    enabled: bool,
    severity: Severity,
    options: HashMap<String, toml::Value>,
}
impl Eq for LintConfig {}

impl LintConfig {
    #[must_use]
    pub fn error() -> Self {
        Self {
            enabled: true,
            severity: Severity::Error,
            options: HashMap::new(),
        }
    }

    #[must_use]
    pub fn warning() -> Self {
        Self {
            enabled: true,
            severity: Severity::Warning,
            options: HashMap::new(),
        }
    }

    #[must_use]
    pub fn help() -> Self {
        Self {
            enabled: true,
            severity: Severity::Help,
            options: HashMap::new(),
        }
    }

    #[must_use]
    pub const fn new(severity: Severity, options: HashMap<String, toml::Value>) -> Self {
        Self {
            severity,
            options,
            enabled: true,
        }
    }

    #[must_use]
    pub fn with_options(self, options: HashMap<String, toml::Value>) -> Self {
        Self { options, ..self }
    }

    #[must_use]
    pub fn with_enabled(self, enabled: bool) -> Self {
        Self { enabled, ..self }
    }

    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    #[must_use]
    pub fn option(&self, key: &str) -> Option<&toml::Value> {
        self.options.get(key)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct LintConfigOverride {
    enabled: Option<bool>,
    severity: Option<Severity>,
    options: HashMap<String, toml::Value>,
}
impl Eq for LintConfigOverride {}

impl LintConfigOverride {
    #[must_use]
    pub const fn enabled(&self) -> Option<bool> {
        self.enabled
    }

    #[must_use]
    pub const fn severity(&self) -> Option<Severity> {
        self.severity
    }

    #[must_use]
    pub fn option(&self, key: &str) -> Option<&toml::Value> {
        self.options.get(key)
    }

    #[must_use]
    pub fn apply(&self, config: LintConfig) -> LintConfig {
        let mut new = config;
        if let Some(enabled) = self.enabled {
            new.enabled = enabled;
        }
        if let Some(severity) = self.severity {
            new.severity = severity;
        }
        new.options.extend(self.options.clone());
        new
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct LintSectionFile {
    pub config: Option<HashMap<String, LintConfigFile>>,
    pub sqf: Option<HashMap<String, LintConfigFile>>,
}

impl From<LintSectionFile> for LintGroupConfig {
    fn from(file: LintSectionFile) -> Self {
        Self {
            config: file
                .config
                .unwrap_or_default()
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            sqf: file
                .sqf
                .unwrap_or_default()
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LintConfigFile {
    Enabled(bool),
    Severity(Severity),
    Full(LintConfigOverride),
}

impl Default for LintConfigFile {
    fn default() -> Self {
        Self::Enabled(true)
    }
}

impl From<LintConfigFile> for LintConfigOverride {
    fn from(file: LintConfigFile) -> Self {
        match file {
            LintConfigFile::Enabled(enabled) => Self {
                enabled: Some(enabled),
                severity: None,
                options: HashMap::new(),
            },
            LintConfigFile::Severity(severity) => Self {
                enabled: None,
                severity: Some(severity),
                options: HashMap::new(),
            },
            LintConfigFile::Full(config) => config,
        }
    }
}
