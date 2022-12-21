#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]

mod error;
use std::fmt::{Display, Formatter};

pub use error::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// which just had to be different from Semver for some reason
/// Arma mod version format
/// Examples of valid version:
/// - 1.0.0.0-d1a631b1
/// - 1.3.24.2452-1a2b3c4d
/// - 1.2.42-1a2b3c4d
/// - 1.2.42.2452
/// - 1.2.42
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    build: Option<u32>,
    hash: Option<String>,
}

impl Version {
    /// Read a version from a `script_version.hpp` files using macros
    ///
    /// ```hpp
    /// #define MAJOR 3
    /// #define MINOR 15
    /// #define PATCHLVL 2
    /// #define BUILD 69
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the file does not contain the correct macros
    pub fn try_from_script_version(version: &str) -> Result<Self, Error> {
        let lines = version.lines().map(str::trim).collect::<Vec<_>>();
        Ok(Self {
            major: Self::extract_version(&lines, "MAJOR")?,
            minor: Self::extract_version(&lines, "MINOR")?,
            patch: Self::extract_version(&lines, "PATCH")?,
            build: Self::extract_version(&lines, "BUILD").ok(),
            hash: None,
        })
    }

    fn extract_version(lines: &[&str], component: &str) -> Result<u32, Error> {
        let error = match component {
            "MAJOR" => Error::ExpectedMajor,
            "MINOR" => Error::ExpectedMinor,
            "PATCH" => Error::ExpectedPatch,
            "BUILD" => Error::ExpectedBuild,
            _ => unreachable!(),
        };
        let line = lines
            .iter()
            .find(|line| line.starts_with(&format!("#define {component}")))
            .ok_or_else(|| error.clone())?;
        // remove comment
        let component = line
            .split_once("//")
            .unwrap_or((line, ""))
            .0
            .trim()
            .rsplit_once(' ')
            .ok_or(error)?;
        component
            .1
            .parse::<u32>()
            .map_err(|_| Error::InvalidComponent(component.1.to_string()))
    }
}

impl TryFrom<&str> for Version {
    type Error = Error;

    fn try_from(version: &str) -> Result<Self, Self::Error> {
        let mut parts = version.split('-');
        let mut version = parts.next().unwrap().split('.');
        let Some(major) = version.next() else {
            return Err(Error::ExpectedMajor);
        };
        let Ok(major) = major.parse() else {
            return Err(Error::InvalidComponent(major.to_string()));
        };
        let Some(minor) = version.next() else {
            return Err(Error::ExpectedMinor);
        };
        let Ok(minor) = minor.parse() else {
            return Err(Error::InvalidComponent(minor.to_string()));
        };
        let Some(patch) = version.next() else {
            return Err(Error::ExpectedPatch);
        };
        let Ok(patch) = patch.parse() else {
            return Err(Error::InvalidComponent(patch.to_string()));
        };
        let build = version.next().map(|build| {
            build
                .parse::<u32>()
                .map_err(|_| Error::InvalidComponent(build.to_string()))
        });
        let build = if let Some(build) = build {
            Some(build?)
        } else {
            None
        };
        let hash = parts.next().map(std::string::ToString::to_string);
        Ok(Self {
            major,
            minor,
            patch,
            build,
            hash,
        })
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut version = format!("{}.{}.{}", self.major, self.minor, self.patch);
        if let Some(build) = self.build {
            version.push_str(&format!(".{build}"));
        }
        if let Some(hash) = &self.hash {
            version.push_str(&format!("-{hash}"));
        }
        serializer.serialize_str(&version)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let version = String::deserialize(deserializer)?;
        Self::try_from(version.as_str()).map_err(serde::de::Error::custom)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(build) = self.build {
            write!(f, ".{build}")?;
        }
        if let Some(hash) = &self.hash {
            write!(f, "-{hash}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = Version::try_from("1.0.0.0-d1a631b1").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
        assert_eq!(version.build, Some(0));
        assert_eq!(version.hash, Some("d1a631b1".to_string()));
    }

    #[test]
    fn test_version_no_build() {
        let version = Version::try_from("1.2.42-1a2b3c4d").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 42);
        assert_eq!(version.build, None);
        assert_eq!(version.hash, Some("1a2b3c4d".to_string()));
    }

    #[test]
    fn test_version_no_hash() {
        let version = Version::try_from("1.2.42.2452").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 42);
        assert_eq!(version.build, Some(2452));
        assert_eq!(version.hash, None);
    }

    #[test]
    fn test_version_no_build_no_hash() {
        let version = Version::try_from("1.2.42").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 42);
        assert_eq!(version.build, None);
        assert_eq!(version.hash, None);
    }

    #[test]
    fn test_version_invalid_component() {
        let version = Version::try_from("1.2.a");
        assert!(version.is_err());
        assert_eq!(
            version.unwrap_err(),
            Error::InvalidComponent("a".to_string())
        );
    }

    #[test]
    fn test_version_missing_minor() {
        let version = Version::try_from("1");
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), Error::ExpectedMinor);
    }

    #[test]
    fn test_version_missing_patch() {
        let version = Version::try_from("1.2");
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), Error::ExpectedPatch);
    }

    #[test]
    fn test_script_version() {
        let content = r#"
            #define MAJOR 1
            #define MINOR 2
            #define PATCH 3
            #define BUILD 4
        "#;
        let version = Version::try_from_script_version(content).unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.build, Some(4));

        assert_eq!(version.hash, None);
    }

    #[test]
    fn test_script_version_comment() {
        let content = r#"
            #define MAJOR 1
            #define MINOR 2
            #define PATCHLVL 3 // some comment
            #define BUILD 4
        "#;
        let version = Version::try_from_script_version(content).unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.build, Some(4));
        assert_eq!(version.hash, None);
    }

    #[test]
    fn test_script_version_no_build() {
        let content = r#"
            #define MAJOR 1
            #define MINOR 2
            #define PATCH 3
        "#;
        let version = Version::try_from_script_version(content).unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.build, None);
        assert_eq!(version.hash, None);
    }

    #[test]
    fn test_script_version_invalid_component() {
        let content = r#"
            #define MAJOR 1
            #define MINOR 2
            #define PATCHLVL a
        "#;
        let version = Version::try_from_script_version(content);
        assert!(version.is_err());
        assert_eq!(
            version.unwrap_err(),
            Error::InvalidComponent("a".to_string())
        );
    }

    #[test]
    fn test_script_version_missing_minor() {
        let content = r#"
            #define MAJOR 1
        "#;
        let version = Version::try_from_script_version(content);
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), Error::ExpectedMinor);
    }

    #[test]
    fn test_script_version_missing_patch() {
        let content = r#"
            #define MAJOR 1
            #define MINOR 2
        "#;
        let version = Version::try_from_script_version(content);
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), Error::ExpectedPatch);
    }

    #[test]
    fn test_script_version_missing_major() {
        let content = r#"
            #define MINOR 2
            #define PATCH 3
        "#;
        let version = Version::try_from_script_version(content);
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), Error::ExpectedMajor);
    }
}
