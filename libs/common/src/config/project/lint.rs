use std::collections::HashMap;

use codespan_reporting::diagnostic::Severity;
use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Lint group config
pub struct LintGroupConfig {
    config: HashMap<String, LintConfigOverride>,
    sqf: HashMap<String, LintConfigOverride>,
    stringtables: HashMap<String, LintConfigOverride>,
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

    #[must_use]
    /// Get the stringtables lints
    pub const fn stringtables(&self) -> &HashMap<String, LintConfigOverride> {
        &self.stringtables
    }

    pub fn is_empty(&self) -> bool {
        self.config.is_empty() && self.sqf.is_empty() && self.stringtables.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LintEnabled {
    Enabled,
    Disabled,
    /// Only enables the lint if --pedantic is set
    Pedantic,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq)]
pub struct LintConfig {
    enabled: LintEnabled,
    severity: Severity,
    minimum_severity: Severity,
    options: HashMap<String, toml::Value>,
}
impl Eq for LintConfig {}

impl LintConfig {
    #[must_use]
    pub fn fatal() -> Self {
        Self {
            enabled: LintEnabled::Enabled,
            severity: Severity::Error,
            minimum_severity: Severity::Error,
            options: HashMap::new(),
        }
    }

    #[must_use]
    pub fn error() -> Self {
        Self {
            enabled: LintEnabled::Enabled,
            severity: Severity::Error,
            minimum_severity: Severity::Warning,
            options: HashMap::new(),
        }
    }

    #[must_use]
    pub fn warning() -> Self {
        Self {
            enabled: LintEnabled::Enabled,
            severity: Severity::Warning,
            minimum_severity: Severity::Help,
            options: HashMap::new(),
        }
    }

    #[must_use]
    pub fn help() -> Self {
        Self {
            enabled: LintEnabled::Enabled,
            severity: Severity::Help,
            minimum_severity: Severity::Help,
            options: HashMap::new(),
        }
    }

    #[must_use]
    pub const fn new(severity: Severity, options: HashMap<String, toml::Value>) -> Self {
        Self {
            severity,
            minimum_severity: severity,
            options,
            enabled: LintEnabled::Enabled,
        }
    }

    #[must_use]
    pub fn with_options(self, options: HashMap<String, toml::Value>) -> Self {
        Self { options, ..self }
    }

    #[must_use]
    pub fn with_enabled(self, enabled: LintEnabled) -> Self {
        Self { enabled, ..self }
    }

    #[must_use]
    pub const fn enabled(&self) -> LintEnabled {
        self.enabled
    }

    #[must_use]
    pub fn with_minimum_severity(self, severity: Severity) -> Self {
        Self {
            minimum_severity: severity,
            ..self
        }
    }

    #[must_use]
    pub const fn minimum_severity(&self) -> Severity {
        self.minimum_severity
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

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LintConfigEnabled {
    Bool(bool),
    String(String),
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize)]
pub struct LintConfigOverride {
    enabled: Option<LintConfigEnabled>,
    severity: Option<Severity>,
    #[serde(default)]
    options: HashMap<String, toml::Value>,
}
impl Eq for LintConfigOverride {}

impl LintConfigOverride {
    #[must_use]
    pub fn enabled(&self) -> Option<LintEnabled> {
        match self.enabled {
            Some(LintConfigEnabled::Bool(true)) => Some(LintEnabled::Enabled),
            Some(LintConfigEnabled::Bool(false)) => Some(LintEnabled::Disabled),
            Some(LintConfigEnabled::String(ref s)) if s == "pedantic" => {
                Some(LintEnabled::Pedantic)
            }
            _ => None,
        }
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
        if let Some(enabled) = self.enabled() {
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
    pub stringtables: Option<HashMap<String, LintConfigFile>>,
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
            stringtables: file
                .stringtables
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
    Severity(Severity),
    Full(LintConfigOverride),
    Enabled(LintConfigEnabled),
}

impl Default for LintConfigFile {
    fn default() -> Self {
        Self::Enabled(LintConfigEnabled::Bool(true))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled() {
        let toml = "
[sqf]
example = false
";
        let file: LintSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = LintGroupConfig::from(file);
        assert_eq!(
            config
                .sqf()
                .get("example")
                .expect("example exists")
                .enabled(),
            Some(LintEnabled::Disabled)
        );
    }

    #[test]
    fn severity() {
        let toml = r#"
[sqf]
example = "Warning"
"#;
        let file: LintSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = LintGroupConfig::from(file);
        assert_eq!(
            config
                .sqf()
                .get("example")
                .expect("example exists")
                .severity(),
            Some(Severity::Warning)
        );
    }

    #[test]
    fn full() {
        let toml = r#"
[sqf.example]
enabled = false
severity = "Warning"
options.test = true
"#;
        let file: LintSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = LintGroupConfig::from(file);
        let example = config.sqf().get("example").expect("example exists");
        assert_eq!(example.enabled(), Some(LintEnabled::Disabled));
        assert_eq!(example.severity(), Some(Severity::Warning));
        assert_eq!(example.option("test"), Some(&toml::Value::Boolean(true)));
    }

    #[test]
    fn empty() {
        let toml = "";
        let file: LintSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = LintGroupConfig::from(file);
        assert!(config.is_empty());
    }

    #[test]
    fn default() {
        let toml = "
[sqf.example]
enabled = true
";
        let file: LintSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = LintGroupConfig::from(file);
        let example = config.sqf().get("example").expect("example exists");
        assert_eq!(example.enabled(), Some(LintEnabled::Enabled));
        assert_eq!(example.severity(), None);
        assert!(example.option("test").is_none());
    }
}
