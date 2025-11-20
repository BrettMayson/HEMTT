use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Clone, Default)]
/// Configuration for the HEMTT preprocess Rhai module
pub struct PreprocessOptions {
    preprocessors: HashMap<String, Vec<String>>,
}

impl PreprocessOptions {
    #[must_use]
    /// Get the list of preprocessors
    pub const fn preprocessors(&self) -> &HashMap<String, Vec<String>> {
        &self.preprocessors
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct PreprocessOptionsFile {
    #[serde(default)]
    preprocessors: HashMap<String, Vec<String>>,
}

impl From<PreprocessOptionsFile> for PreprocessOptions {
    fn from(file: PreprocessOptionsFile) -> Self {
        Self {
            preprocessors: file.preprocessors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocessor_options_file() {
        let toml: &'static str = r#"
        [preprocessors]
        processor1 = ["path1", "path2"]
        processor2 = ["path3"]
        "#;
        let file: PreprocessOptionsFile = toml::from_str(toml).expect("Failed to parse TOML");
        let options: PreprocessOptions = file.into();
        assert_eq!(options.preprocessors(), &{
            let mut map = HashMap::new();
            map.insert(
                "processor1".to_string(),
                vec!["path1".to_string(), "path2".to_string()],
            );
            map.insert("processor2".to_string(), vec!["path3".to_string()]);
            map
        });
    }
}
