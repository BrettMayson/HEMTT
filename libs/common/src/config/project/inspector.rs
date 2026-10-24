use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Clone, Default)]
/// Configuration for inspector options
pub struct InspectorOptions {
    /// variable names to ignore when checking for undefined variables
    vars_to_ignore: Option<Vec<String>>,
    /// function prefixes to check for when calling
    check_function_calls: Vec<String>,
    /// project function prefixes to export to `.hemtt/functions`
    export_functions: Vec<String>,
    /// header regex
    header_regex: String,
    /// header regex for a line
    header_line_regex: String,
}

impl InspectorOptions {
    #[must_use]
    pub fn vars_to_ignore(&self) -> Option<&[String]> {
        self.vars_to_ignore.as_deref()
    }
    #[must_use]
    pub fn with_vars_to_ignore(mut self, value: Option<Vec<String>>) -> Self {
        self.vars_to_ignore = value;
        self
    }
    #[must_use]
    pub fn check_function_calls(&self) -> &[String] {
        &self.check_function_calls
    }
    #[must_use]
    pub fn with_check_function_calls(mut self, value: Vec<String>) -> Self {
        self.check_function_calls = value;
        self
    }
    #[must_use]
    pub fn export_functions(&self) -> &[String] {
        &self.export_functions
    }
    #[must_use]
    pub fn with_export_functions(mut self, value: Vec<String>) -> Self {
        self.export_functions = value;
        self
    }
    #[must_use]
    pub fn header_regex(&self) -> &str {
        &self.header_regex
    }
    #[must_use]
    pub fn with_header_regex(mut self, value: String) -> Self {
        self.header_regex = value;
        self
    }
    #[must_use]
    pub fn header_line_regex(&self) -> &str {
        &self.header_line_regex
    }
    #[must_use]
    pub fn with_header_line_regex(mut self, value: String) -> Self {
        self.header_line_regex = value;
        self
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum VectorOrBoolWildcard {
    Vec(Vec<String>),
    Bool(bool),
}
impl VectorOrBoolWildcard {
    /// false becomes empty vec, true becomes vec with wildcard "*"
    fn to_vec(&self) -> Vec<String> {
        match self {
            Self::Vec(v) => v.iter().map(|s| s.to_lowercase()).collect(),
            Self::Bool(b) => {
                if *b {
                    vec!["*".to_string()]
                } else {
                    vec![]
                }
            }
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct InspectorOptionsFile {
    #[serde(default)]
    vars_to_ignore: Option<Vec<String>>,
    #[serde(default)]
    check_function_calls: Option<VectorOrBoolWildcard>,
    #[serde(default)]
    export_functions: Option<VectorOrBoolWildcard>,
    #[serde(default)]
    header_regex: Option<String>,
    #[serde(default)]
    header_line_regex: Option<String>,
}

impl From<InspectorOptionsFile> for InspectorOptions {
    fn from(file: InspectorOptionsFile) -> Self {
        Self {
            vars_to_ignore: file.vars_to_ignore, // default is None, which will load the CfgFunction vars (_fnc_scriptName...)
            check_function_calls: file
                .check_function_calls
                .unwrap_or(VectorOrBoolWildcard::Bool(false)) // opt-in to check func calls
                .to_vec(),
            export_functions: file
                .export_functions
                .unwrap_or(VectorOrBoolWildcard::Bool(true)) // default all
                .to_vec(),
            header_regex: file.header_regex.unwrap_or_default(), // opt-in to header parsing, either preset or actual regex
            header_line_regex: file.header_line_regex.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_inspector_options_file_1() {
        let toml: &'static str = r"";
        let file: InspectorOptionsFile = toml::from_str(toml).expect("Failed to parse TOML");
        let options: InspectorOptions = file.into();
        assert!(options.vars_to_ignore().is_none());
        assert!(options.check_function_calls().is_empty());
        assert_eq!(options.export_functions(), &vec!["*".to_string()]);
        assert_eq!(options.header_regex, "");
        assert_eq!(options.header_line_regex, "");
    }
    #[test]
    fn test_inspector_options_file_2() {
        let toml: &'static str = r#"
        export_functions = ["abe_berry"]
        check_function_calls = true
        vars_to_ignore = []
        header_regex = "ace"
    	"#;
        let file: InspectorOptionsFile = toml::from_str(toml).expect("Failed to parse TOML");
        let options: InspectorOptions = file.into();
        assert!(options.vars_to_ignore().expect("some").is_empty());
        assert_eq!(options.check_function_calls(), &vec!["*".to_string()]);
        assert_eq!(options.export_functions(), &vec!["abe_berry".to_string()]);
        assert_eq!(options.header_regex, "ace");
        assert_eq!(options.header_line_regex, "");
    }
}
