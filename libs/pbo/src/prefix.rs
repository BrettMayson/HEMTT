//! A prefix file for a PBO
//!
//! It can be a single line defining the prefix, or a key-value pair defining the prefix and extra headers
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

use crate::Error;

/// Files that may be used to contain the prefix, case insensitive, convert to lowercase
pub const FILES: [&str; 3] = ["$pboprefix$", "pboprefix.txt", "$prefix$"];

#[derive(Debug, Clone, PartialEq, Eq)]
/// A prefix for a PBO
pub struct Prefix(String);

impl Prefix {
    /// Read a prefix from a prefix file
    ///
    /// # Errors
    /// If the prefix is invalid
    pub fn new(content: &str, allow_leading_slash: bool) -> Result<Self, Error> {
        let prefix = Self::_from_prefix_file(content, allow_leading_slash)?;
        if prefix.0.contains('/') {
            return Err(Error::InvalidPrefix(prefix.0));
        }
        Ok(prefix)
    }

    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    /// Get the prefix as a string
    pub fn into_inner(self) -> String {
        self.0
    }

    fn _from_prefix_file(content: &str, allow_leading_slash: bool) -> Result<Self, Error> {
        let content = content.trim();
        let line_count = content.lines().count();
        if line_count == 1 && !content.contains('=') {
            if content.starts_with('\\') {
                if allow_leading_slash {
                    return Ok(Self(content.strip_prefix('\\').unwrap().to_string()));
                }
                return Err(Error::InvalidPrefix(content.to_string()));
            }
            return Ok(Self(content.to_string()));
        }
        for line in content.lines() {
            if let Some(split) = line.split_once('=') {
                let key = split.0.trim().to_lowercase();
                if key == "prefix" {
                    let content = split.1.trim();
                    if content.starts_with('\\') {
                        if allow_leading_slash {
                            return Ok(Self(content.strip_prefix('\\').unwrap().to_string()));
                        }
                        return Err(Error::InvalidPrefix(content.to_string()));
                    }
                    return Ok(Self(content.to_string()));
                }
            }
        }
        Err(Error::InvalidPrefix(content.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::Prefix;

    #[test]
    fn just_prefix() {
        let prefix = Prefix::new("z\\test\\addons\\main", false).unwrap();
        assert_eq!(prefix.0, "z\\test\\addons\\main");
        assert!(Prefix::new("z/test/addons/main", false).is_err());
        assert!(Prefix::new("\\z\\test\\addons\\main", false).is_err());
        let prefix = Prefix::new("\\z\\test\\addons\\main", true).unwrap();
        assert_eq!(prefix.0, "z\\test\\addons\\main");
    }

    #[test]
    fn with_key() {
        let prefix = Prefix::new("prefix=z\\test\\addons\\main", false).unwrap();
        assert_eq!(prefix.0, "z\\test\\addons\\main");
        assert!(Prefix::new("prefix=z/test/addons/main", false).is_err());
        assert!(Prefix::new("prefix=\\z\\test\\addons\\main", false).is_err());
        let prefix = Prefix::new("prefix=\\z\\test\\addons\\main", true).unwrap();
        assert_eq!(prefix.0, "z\\test\\addons\\main");
    }

    #[test]
    fn with_keys() {
        let prefix = Prefix::new("prefix=z\\test\\addons\\main\nother=stuff", false).unwrap();
        assert_eq!(prefix.0, "z\\test\\addons\\main");
        assert!(Prefix::new("prefix=z/test/addons/main\nother=stuff", false).is_err());
        assert!(Prefix::new("prefix=\\z\\test\\addons\\main\nother=stuff", false).is_err());
        let prefix = Prefix::new("prefix=\\z\\test\\addons\\main\nother=stuff", true).unwrap();
        assert_eq!(prefix.0, "z\\test\\addons\\main");
    }
}
