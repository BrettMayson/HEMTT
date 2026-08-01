use std::{collections::HashMap, ops::Range};

use codespan_reporting::diagnostic::Severity;
use serde::{Deserialize, Serialize};

/// Loads the toml lints from the lints folder at the given path
///
/// # Errors
/// [`crate::Error::Io`] if the lints folder cannot be read
/// [`crate::Error::Toml`] if a lint file is not valid toml
/// [`crate::Error::TomlLint`] if a lint file is not valid
///
/// # Panics
/// Panics if the path provided is not in the .hemtt folder
pub fn load_toml_lints(path: &std::path::Path) -> Result<Vec<TomlLint>, crate::Error> {
    let lints_dir = path.parent().expect("in .hemtt folder").join("lints");
    if !lints_dir.is_dir() {
        return Ok(Vec::new());
    }
    // read all lints from the lints directory
    let mut lints = Vec::new();
    for entry in fs_err::read_dir(&lints_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
            let source = fs_err::read_to_string(&path)?;
            let lint_file: TomlLintFile = toml::from_str(&source)?;
            let lint = TomlLint::try_from(lint_file)?;
            lints.push(lint);
        }
    }
    Ok(lints)
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TomlLint {
    severity: Option<Severity>,
    note: Option<String>,
    help: Option<String>,
    message: Option<String>,
    label: Option<String>,

    sqf: HashMap<TomlLintSqfTarget, TomlLintDef>,
    config: HashMap<TomlLintConfigTarget, TomlLintDef>,
}

impl TomlLint {
    #[must_use]
    pub const fn severity(&self) -> Option<Severity> {
        self.severity
    }

    #[must_use]
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    #[must_use]
    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    #[must_use]
    pub const fn sqf(&self) -> &HashMap<TomlLintSqfTarget, TomlLintDef> {
        &self.sqf
    }

    #[must_use]
    pub const fn config(&self) -> &HashMap<TomlLintConfigTarget, TomlLintDef> {
        &self.config
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum TomlLintSqfTarget {
    File,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum TomlLintConfigTarget {
    File,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TomlLintDef {
    Patterns(Vec<String>),
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct TomlLintFile {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    severity: Option<Severity>,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    help: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    label: Option<String>,

    #[serde(default)]
    sqf: Option<HashMap<String, TomlLintDefFile>>,
    #[serde(default)]
    config: Option<HashMap<String, TomlLintDefFile>>,
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct TomlLintDefFile {
    #[serde(default)]
    patterns: Option<Vec<String>>,
}

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
/// Errors that can occur while parsing a TOML lint file
pub enum Error {
    #[error("Unknown sqf target '{0}'")]
    UnknownSqfTarget(String),
    #[error("Unknown config target '{0}'")]
    UnknownConfigTarget(String),
    #[error("No patterns defined for file lint")]
    NoPatterns,
    #[error("Invalid regex pattern '{0}'")]
    InvalidRegexPattern(String),
}

impl TryFrom<TomlLintFile> for TomlLint {
    type Error = Error;
    fn try_from(file: TomlLintFile) -> Result<Self, Self::Error> {
        let sqf = file
            .sqf
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| Ok((k.try_into()?, v.try_into()?)))
            .collect::<Result<HashMap<_, _>, Error>>()?;
        let config = file
            .config
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| Ok((k.try_into()?, v.try_into()?)))
            .collect::<Result<HashMap<_, _>, Error>>()?;
        Ok(Self {
            severity: file.severity,
            note: file.note,
            help: file.help,
            message: file.message,
            label: file.label,
            sqf,
            config,
        })
    }
}

impl TryFrom<TomlLintDefFile> for TomlLintDef {
    type Error = Error;
    fn try_from(file: TomlLintDefFile) -> Result<Self, Self::Error> {
        if let Some(patterns) = file.patterns {
            if patterns.is_empty() {
                Err(Error::NoPatterns)
            } else {
                Ok(Self::Patterns(
                    patterns
                        .into_iter()
                        .map(|p| {
                            let _ = regex::Regex::new(&p)
                                .map_err(|_| Error::InvalidRegexPattern(p.clone()))?;
                            Ok(p)
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                ))
            }
        } else {
            Err(Error::NoPatterns)
        }
    }
}

impl TryFrom<String> for TomlLintSqfTarget {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "file" => Ok(Self::File),
            _ => Err(Error::UnknownSqfTarget(value)),
        }
    }
}

impl TryFrom<String> for TomlLintConfigTarget {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "file" => Ok(Self::File),
            _ => Err(Error::UnknownConfigTarget(value)),
        }
    }
}

impl TomlLint {
    #[must_use]
    /// Returns the ranges of the given source that match any of the patterns defined in this lint
    ///
    /// # Panics
    /// Panics if the regex patterns are invalid, which should not happen if the lint was loaded successfully
    pub fn run_file(&self, source: &str, patterns: &[String]) -> Vec<Range<usize>> {
        let mut ranges = Vec::new();
        for pattern in patterns {
            let regex = regex::Regex::new(pattern).expect("Invalid regex pattern in lint");
            for mat in regex.find_iter(source) {
                ranges.push(mat.range());
            }
        }
        ranges
    }
}
