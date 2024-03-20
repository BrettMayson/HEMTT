use std::{ffi::OsStr, fmt::Display, path::PathBuf};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
/// Version of BI's signature
pub enum BISignVersion {
    /// Version 2
    ///
    /// Hashes the following file extensions:
    /// - fxy
    /// - jpg
    /// - lip
    /// - ogg
    /// - p3d
    /// - paa
    /// - pac
    /// - png
    /// - rtm
    /// - rvmat
    /// - tga
    /// - wrp
    /// - wss
    V2,
    #[default]
    /// Version 3
    ///
    /// Hashes the following file extensions:
    /// - bikb
    /// - cfg
    /// - ext
    /// - fsm
    /// - h
    /// - hpp
    /// - inc
    /// - sqf
    /// - sqfc
    /// - sqm
    /// - sqs
    V3,
}

impl BISignVersion {
    #[must_use]
    /// Should a file be hashed?
    pub fn should_hash_file(&self, name: &str) -> bool {
        let path = PathBuf::from(name);
        let ext = path.extension().unwrap_or_else(|| OsStr::new(""));
        match self {
            Self::V2 => [
                OsStr::new("fxy"),
                OsStr::new("jpg"),
                OsStr::new("lip"),
                OsStr::new("ogg"),
                OsStr::new("p3d"),
                OsStr::new("paa"),
                OsStr::new("pac"),
                OsStr::new("png"),
                OsStr::new("rtm"),
                OsStr::new("rvmat"),
                OsStr::new("tga"),
                OsStr::new("wrp"),
                OsStr::new("wss"),
            ]
            .contains(&ext),
            Self::V3 => [
                OsStr::new("bikb"),
                OsStr::new("cfg"),
                OsStr::new("ext"),
                OsStr::new("fsm"),
                OsStr::new("h"),
                OsStr::new("hpp"),
                OsStr::new("inc"),
                OsStr::new("sqf"),
                OsStr::new("sqfc"),
                OsStr::new("sqm"),
                OsStr::new("sqs"),
            ]
            .contains(&ext),
        }
    }

    #[must_use]
    /// Get the nothing string for the version
    pub const fn nothing(&self) -> &str {
        match self {
            Self::V2 => "nothing",
            Self::V3 => "gnihton",
        }
    }
}

impl From<BISignVersion> for u32 {
    fn from(v: BISignVersion) -> Self {
        match v {
            BISignVersion::V2 => 0x02,
            BISignVersion::V3 => 0x03,
        }
    }
}

impl Serialize for BISignVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(u32::from(*self))
    }
}

impl<'de> Deserialize<'de> for BISignVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = u32::deserialize(deserializer)?;
        match v {
            0x02 => Ok(Self::V2),
            0x03 => Ok(Self::V3),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid BISignVersion: {v}"
            ))),
        }
    }
}

impl Display for BISignVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V2 => write!(f, "V2"),
            Self::V3 => write!(f, "V3"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    #[test]
    fn serialize_and_deserialize() {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Test {
            version: super::BISignVersion,
        }

        let v = Test {
            version: super::BISignVersion::V2,
        };
        let s = toml::to_string(&v).unwrap();
        assert_eq!(s, "version = 2\n");
        let v: Test = toml::from_str(&s).unwrap();
        assert_eq!(v.version, super::BISignVersion::V2);

        let v = Test {
            version: super::BISignVersion::V3,
        };
        let s = toml::to_string(&v).unwrap();
        assert_eq!(s, "version = 3\n");
        let v: Test = toml::from_str(&s).unwrap();
        assert_eq!(v.version, super::BISignVersion::V3);
    }

    #[test]
    fn should_hash_file() {
        assert!(super::BISignVersion::V2.should_hash_file("test.paa"));
        assert!(!super::BISignVersion::V2.should_hash_file("test.sqf"));
        assert!(super::BISignVersion::V3.should_hash_file("test.sqf"));
        assert!(!super::BISignVersion::V3.should_hash_file("test.paa"));
    }
}
