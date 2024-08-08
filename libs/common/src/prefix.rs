//! A prefix file for a PBO
//!
//! It can be a single line defining the prefix, or a key-value pair defining the prefix and extra properties
//!
//! # Examples
//!
//! ```text
//! \z\hemtt\addons\main
//! ```
//!
//! ```text
//! prefix=\z\hemtt\addons\main
//! extra=header
//! ```

use std::{fmt::Display, path::PathBuf};

/// Files that may be used to contain the prefix, case insensitive, convert to lowercase
pub const FILES: [&str; 4] = [
    "$pboprefix$",
    "$pboprefix$.txt",
    "pboprefix.txt",
    "$prefix$",
];

#[derive(Debug, Clone, PartialEq, Eq)]
/// A prefix for a PBO
pub struct Prefix(Vec<String>);

impl Prefix {
    /// Read a prefix from a prefix file
    ///
    /// # Errors
    /// If the prefix is invalid
    pub fn new(content: &str) -> Result<Self, Error> {
        let content = content.trim();
        if content.is_empty() {
            return Err(Error::Empty);
        }
        if content.contains('/') {
            return Err(Error::InvalidPrefix(content.to_string()));
        }
        let line_count = content.lines().count();
        if line_count == 1 && !content.contains('=') {
            if content.starts_with('\\') {
                return Err(Error::InvalidPrefix(content.to_string()));
            }
            return Ok(Self(
                content
                    .split('\\')
                    .map(std::string::ToString::to_string)
                    .collect(),
            ));
        }
        for line in content.lines() {
            if let Some(split) = line.split_once('=') {
                let key = split.0.trim().to_lowercase();
                if key == "prefix" {
                    let content = split.1.trim();
                    if content.starts_with('\\') {
                        return Err(Error::InvalidPrefix(content.to_string()));
                    }
                    return Ok(Self(
                        content
                            .split('\\')
                            .map(std::string::ToString::to_string)
                            .collect(),
                    ));
                }
            }
        }
        Err(Error::InvalidPrefix(content.to_string()))
    }

    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    /// Get the parts of the prefix
    pub fn into_inner(self) -> Vec<String> {
        self.0
    }

    #[must_use]
    /// Get the main prefix
    pub fn main_prefix(&self) -> &str {
        &self.0[0]
    }

    #[must_use]
    /// Get the mod prefix
    pub fn mod_prefix(&self) -> Option<&str> {
        self.0.get(1).map(std::string::String::as_str)
    }

    #[must_use]
    /// Get the prefix as a pathbuf
    pub fn as_pathbuf(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.0[0]);
        for part in &self.0[1..] {
            path.push(part);
        }
        path
    }
}

impl Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("\\"))
    }
}

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
/// An error that can occur when parsing a prefix
pub enum Error {
    #[error("Prefix can not be empty")]
    /// Prefix can not be empty
    Empty,
    #[error("Invalid prefix: {0}")]
    /// Invalid prefix
    InvalidPrefix(String),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::Prefix;

    #[test]
    fn just_prefix() {
        let prefix = Prefix::new("z\\test\\addons\\main").unwrap();
        assert_eq!(prefix.to_string(), "z\\test\\addons\\main");
        assert_eq!(prefix.main_prefix(), "z");
        assert_eq!(prefix.mod_prefix().unwrap(), "test");
        assert!(Prefix::new("z/test/addons/main").is_err());
        assert!(Prefix::new("\\z\\test\\addons\\main").is_err());
    }

    #[test]
    fn with_key() {
        let prefix = Prefix::new("prefix=z\\test\\addons\\main").unwrap();
        assert_eq!(prefix.to_string(), "z\\test\\addons\\main");
        assert_eq!(prefix.main_prefix(), "z");
        assert_eq!(prefix.mod_prefix().unwrap(), "test");
        assert!(Prefix::new("prefix=z/test/addons/main").is_err());
        assert!(Prefix::new("prefix=\\z\\test\\addons\\main").is_err());
    }

    #[test]
    fn with_keys() {
        let prefix = Prefix::new("prefix=z\\test\\addons\\main\nother=stuff").unwrap();
        assert_eq!(prefix.to_string(), "z\\test\\addons\\main");
        assert_eq!(prefix.main_prefix(), "z");
        assert_eq!(prefix.mod_prefix().unwrap(), "test");
        assert!(Prefix::new("prefix=z/test/addons/main\nother=stuff").is_err());
        assert!(Prefix::new("prefix=\\z\\test\\addons\\main\nother=stuff").is_err());
    }

    #[test]
    fn single() {
        let prefix = Prefix::new("test").unwrap();
        assert_eq!(prefix.to_string(), "test");
        assert_eq!(prefix.main_prefix(), "test");
        assert!(prefix.mod_prefix().is_none());
    }
}
