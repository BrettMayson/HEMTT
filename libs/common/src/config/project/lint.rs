use std::collections::HashMap;

use codespan_reporting::diagnostic::Severity;
use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Lint group config
pub struct LintGroupConfig {
    /// Lints for config
    config: HashMap<String, LintConfig>,
}

impl LintGroupConfig {
    #[must_use]
    /// Get the lints
    pub const fn config(&self) -> &HashMap<String, LintConfig> {
        &self.config
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
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
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct LintSectionFile {
    pub lints: HashMap<String, LintConfig>,
}

impl From<LintSectionFile> for LintGroupConfig {
    fn from(file: LintSectionFile) -> Self {
        Self { config: file.lints }
    }
}
