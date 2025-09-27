use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Clone, Default)]
/// Configuration for the HEMTT preprocessor
pub struct PreprocessorOptions {
    runtime_macros: bool,
}

impl PreprocessorOptions {
    #[must_use]
    /// Should the preprocessor ignore all runtime macros?
    pub const fn runtime_macros(&self) -> bool {
        self.runtime_macros
    }

    #[must_use]
    pub fn with_runtime_macros(mut self, value: bool) -> Self {
        self.runtime_macros = value;
        self
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct PreprocessorOptionsFile {
    #[serde(default)]
    runtime_macros: Option<bool>,
}

impl From<PreprocessorOptionsFile> for PreprocessorOptions {
    fn from(file: PreprocessorOptionsFile) -> Self {
        Self {
            runtime_macros: file.runtime_macros.unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocessor_options() {
        let options = PreprocessorOptions::default().with_runtime_macros(true);
        assert!(options.runtime_macros());
    }

    #[test]
    fn test_preprocessor_options_file() {
        let toml: &'static str = r"
        runtime_macros = true
        ";
        let file: PreprocessorOptionsFile = toml::from_str(toml).expect("Failed to parse TOML");
        let options: PreprocessorOptions = file.into();
        assert!(options.runtime_macros());
    }
}
