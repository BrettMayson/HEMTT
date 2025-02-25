//! Versioning for Arma mods

use std::fmt::{Display, Formatter, Write};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

// which just had to be different from Semver for some reason
/// Arma mod version format
/// Examples of valid version:
/// - 1.0.0.0-d1a631b1
/// - 1.3.24.2452-1a2b3c4d
/// - 1.2.42-1a2b3c4d
/// - 1.2.42.2452
/// - 1.2.42
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    build: Option<u32>,
    hash: Option<String>,
}

impl Version {
    /// Create a new version
    #[must_use]
    pub const fn new(major: u32, minor: u32, patch: u32, build: Option<u32>) -> Self {
        Self {
            major,
            minor,
            patch,
            build,
            hash: None,
        }
    }

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

    /// Set the build number
    pub fn set_build(&mut self, build: impl Into<String>) {
        self.hash = Some(build.into());
    }

    /// Major version number
    #[must_use]
    pub const fn major(&self) -> u32 {
        self.major
    }

    /// Minor version number
    #[must_use]
    pub const fn minor(&self) -> u32 {
        self.minor
    }

    /// Patch version number
    #[must_use]
    pub const fn patch(&self) -> u32 {
        self.patch
    }

    /// Build number
    #[must_use]
    pub const fn build(&self) -> Option<u32> {
        self.build
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
        if version.is_empty() {
            return Err(Error::UnknownVersion);
        }
        let mut parts = version.split('-');
        let mut version = parts
            .next()
            .expect("should have something to attempt to parse")
            .split('.');
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

impl From<f32> for Version {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn from(version: f32) -> Self {
        let integer = version as u32;
        let decimal = (version.fract() * 100.0).round() as u32;
        Self::new(integer, decimal, 0, None)
    }
}

/// Compare two versions
impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.major != other.major {
            return Some(self.major.cmp(&other.major));
        }
        if self.minor != other.minor {
            return Some(self.minor.cmp(&other.minor));
        }
        if self.patch != other.patch {
            return Some(self.patch.cmp(&other.patch));
        }
        if self.build.is_none() && other.build.is_none() {
            return Some(std::cmp::Ordering::Equal);
        }
        if self.build.is_none() {
            return Some(std::cmp::Ordering::Less);
        }
        if other.build.is_none() {
            return Some(std::cmp::Ordering::Greater);
        }
        self.build.partial_cmp(&other.build)
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut version = format!("{}.{}.{}", self.major, self.minor, self.patch);
        if let Some(build) = self.build {
            write!(version, ".{build}").unwrap();
        }
        if let Some(hash) = &self.hash {
            write!(version, "-{hash}").unwrap();
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

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
/// Errors that can occur while parsing a version
pub enum Error {
    #[error("Unknown Version")]
    /// HEMTT was unable to determine the project version
    UnknownVersion,

    #[error("Expected Major")]
    /// HEMTT exoected but did not find a major version
    ExpectedMajor,

    #[error("Expected Minor")]
    /// HEMTT exoected but did not find a minor version
    ExpectedMinor,

    #[error("Expected Patch")]
    /// HEMTT exoected but did not find a patch version
    ExpectedPatch,

    #[error("Expected Build")]
    /// HEMTT exoected but did not find a build version
    ExpectedBuild,

    #[error("Not a valid component: {0}")]
    /// HEMTT found an invalid version component
    InvalidComponent(String),

    #[error("Version definition conflict, can define either a path or components")]
    /// HEMTT found a conflict between a version path and version components
    VersionPathConflict,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn version() {
        let version = Version::try_from("1.0.0.0-d1a631b1").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
        assert_eq!(version.build, Some(0));
        assert_eq!(version.hash, Some("d1a631b1".to_string()));
    }

    #[test]
    fn version_no_build() {
        let version = Version::try_from("1.2.42-1a2b3c4d").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 42);
        assert_eq!(version.build, None);
        assert_eq!(version.hash, Some("1a2b3c4d".to_string()));
    }

    #[test]
    fn version_hemtt_local() {
        let version = Version::try_from("1.10.1-local-debug").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 10);
        assert_eq!(version.patch, 1);
        assert_eq!(version.build, None);
        assert_eq!(version.hash, Some("local".to_string()));
    }

    #[test]
    fn version_no_hash() {
        let version = Version::try_from("1.2.42.2452").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 42);
        assert_eq!(version.build, Some(2452));
        assert_eq!(version.hash, None);
    }

    #[test]
    fn version_no_build_no_hash() {
        let version = Version::try_from("1.2.42").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 42);
        assert_eq!(version.build, None);
        assert_eq!(version.hash, None);
    }

    #[test]
    fn version_set_build() {
        let mut version = Version::try_from("1.2.42").unwrap();
        version.set_build("1a2b3c4d");
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 42);
        assert_eq!(version.build, None);
        assert_eq!(version.hash, Some("1a2b3c4d".to_string()));
    }

    #[test]
    fn version_getters() {
        let version = Version::try_from("1.2.42-1a2b3c4d").unwrap();
        assert_eq!(version.major(), 1);
        assert_eq!(version.minor(), 2);
        assert_eq!(version.patch(), 42);
        assert_eq!(version.build(), None);
    }

    #[test]
    fn version_invalid_component() {
        let version = Version::try_from("1.2.a");
        assert!(version.is_err());
        assert_eq!(
            version.unwrap_err(),
            Error::InvalidComponent("a".to_string())
        );
    }

    #[test]
    fn version_empty() {
        let version = Version::try_from("");
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), Error::UnknownVersion);
    }

    #[test]
    fn version_dot() {
        let version = Version::try_from(".");
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), Error::InvalidComponent(String::new()));
    }

    #[test]
    fn version_invalid_major() {
        let version = Version::try_from("a.2.3");
        assert!(version.is_err());
        assert_eq!(
            version.unwrap_err(),
            Error::InvalidComponent("a".to_string())
        );
    }

    #[test]
    fn version_invalid_minor() {
        let version = Version::try_from("1.a.3");
        assert!(version.is_err());
        assert_eq!(
            version.unwrap_err(),
            Error::InvalidComponent("a".to_string())
        );
    }

    #[test]
    fn version_invalid_patch() {
        let version = Version::try_from("1.2.a");
        assert!(version.is_err());
        assert_eq!(
            version.unwrap_err(),
            Error::InvalidComponent("a".to_string())
        );
    }

    #[test]
    fn version_invalid_build() {
        let version = Version::try_from("1.2.3.a");
        assert!(version.is_err());
        assert_eq!(
            version.unwrap_err(),
            Error::InvalidComponent("a".to_string())
        );
    }

    #[test]
    fn version_missing_major() {
        let version = Version::try_from(".2.3");
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), Error::InvalidComponent(String::new()));
    }

    #[test]
    fn version_missing_minor() {
        let version = Version::try_from("1");
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), Error::ExpectedMinor);
    }

    #[test]
    fn version_missing_patch() {
        let version = Version::try_from("1.2");
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), Error::ExpectedPatch);
    }

    #[test]
    fn script_version() {
        let content = r"
            #define MAJOR 1
            #define MINOR 2
            #define PATCH 3
            #define BUILD 4
        ";
        let version = Version::try_from_script_version(content).unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.build, Some(4));

        assert_eq!(version.hash, None);
    }

    #[test]
    fn script_version_comment() {
        let content = r"
            #define MAJOR 1
            #define MINOR 2
            #define PATCHLVL 3 // some comment
            #define BUILD 4
        ";
        let version = Version::try_from_script_version(content).unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.build, Some(4));
        assert_eq!(version.hash, None);
    }

    #[test]
    fn script_version_no_build() {
        let content = r"
            #define MAJOR 1
            #define MINOR 2
            #define PATCH 3
        ";
        let version = Version::try_from_script_version(content).unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.build, None);
        assert_eq!(version.hash, None);
    }

    #[test]
    fn script_version_invalid_component() {
        let content = r"
            #define MAJOR 1
            #define MINOR 2
            #define PATCHLVL a
        ";
        let version = Version::try_from_script_version(content);
        assert!(version.is_err());
        assert_eq!(
            version.unwrap_err(),
            Error::InvalidComponent("a".to_string())
        );
    }

    #[test]
    fn script_version_missing_minor() {
        let content = r"
            #define MAJOR 1
        ";
        let version = Version::try_from_script_version(content);
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), Error::ExpectedMinor);
    }

    #[test]
    fn script_version_missing_patch() {
        let content = r"
            #define MAJOR 1
            #define MINOR 2
        ";
        let version = Version::try_from_script_version(content);
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), Error::ExpectedPatch);
    }

    #[test]
    fn script_version_missing_major() {
        let content = r"
            #define MINOR 2
            #define PATCH 3
        ";
        let version = Version::try_from_script_version(content);
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), Error::ExpectedMajor);
    }

    #[test]
    fn float() {
        let version = Version::from(1.2);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 20);
        assert_eq!(version.patch, 0);
        assert_eq!(version.build, None);
        assert_eq!(version.hash, None);
        let version = Version::from(1.16);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 16);
        assert_eq!(version.patch, 0);
        assert_eq!(version.build, None);
    }

    #[test]
    fn ordering() {
        // Major
        assert!(Version::from(2.0) > Version::from(1.0));
        assert!(Version::from(1.0) < Version::from(2.0));
        // Minor
        assert!(Version::from(1.2) > Version::from(1.1));
        assert!(Version::from(1.1) < Version::from(1.2));
        // Patch
        assert!(Version::new(1, 1, 2, None) > Version::new(1, 1, 1, None));
        assert!(Version::new(1, 1, 1, None) < Version::new(1, 1, 2, None));
        // Build
        assert!(Version::new(1, 1, 1, Some(2)) > Version::new(1, 1, 1, Some(1)));
        assert!(Version::new(1, 1, 1, Some(1)) < Version::new(1, 1, 1, Some(2)));
        assert!(Version::new(1, 1, 1, Some(1)) > Version::new(1, 1, 1, None));
        assert!(Version::new(1, 1, 1, None) < Version::new(1, 1, 1, Some(1)));
    }

    #[test]
    fn serialize_and_deserialize() {
        #[derive(Serialize, Deserialize)]
        struct Wrapper {
            version: Version,
        }
        for v in ["1.2.3", "1.2.3.4", "1.2.3.4-test", "1.2.3-test"] {
            let version = Version::try_from(v).unwrap();
            let serialized = toml::to_string(&Wrapper {
                version: version.clone(),
            })
            .unwrap();
            let deserialized: Wrapper = toml::from_str(&serialized).unwrap();
            assert_eq!(deserialized.version, version);
        }
    }
}
