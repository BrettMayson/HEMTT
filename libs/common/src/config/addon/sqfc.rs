use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Default)]
/// SQFC (compiled SQF) config
pub struct SqfcConfig {
    /// Files that are never compiled to SQFC, they are still linted
    exclude: Vec<String>,
    /// Files that are temporarily not included as SQFC
    ///
    /// HEMTT still attempts to compile them, and warns when it succeeds
    /// so the entry can be removed
    exclude_temporary: Vec<String>,
}

impl SqfcConfig {
    #[must_use]
    /// Files that are never compiled to SQFC, they are still linted
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }

    #[must_use]
    /// Files that are temporarily not included as SQFC
    pub const fn exclude_temporary(&self) -> &Vec<String> {
        &self.exclude_temporary
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Section of the addon.toml file for SQFC
pub struct SqfcSectionFile {
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default)]
    exclude_temporary: Vec<String>,
}

impl From<SqfcSectionFile> for SqfcConfig {
    fn from(file: SqfcSectionFile) -> Self {
        Self {
            exclude: file.exclude,
            exclude_temporary: file.exclude_temporary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_defined() {
        let toml = r#"
exclude = ["functions/fnc_version.sqf"]
exclude_temporary = ["functions/fnc_data.sqf"]
"#;
        let file: SqfcSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = SqfcConfig::from(file);
        assert_eq!(config.exclude(), &["functions/fnc_version.sqf".to_string()]);
        assert_eq!(
            config.exclude_temporary(),
            &["functions/fnc_data.sqf".to_string()]
        );
    }

    #[test]
    fn empty() {
        let file: SqfcSectionFile = toml::from_str("").expect("failed to deserialize");
        let config = SqfcConfig::from(file);
        assert_eq!(config.exclude(), &[] as &[String; 0]);
        assert_eq!(config.exclude_temporary(), &[] as &[String; 0]);
    }
}
