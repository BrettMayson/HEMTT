use std::{ffi::OsStr, path::PathBuf};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Copy, Clone, Debug, Default)]
#[allow(clippy::module_name_repetitions)]
pub enum BISignVersion {
    V2,
    #[default]
    V3,
}

impl BISignVersion {
    #[must_use]
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
                OsStr::new("sqf"),
                OsStr::new("inc"),
                OsStr::new("bikb"),
                OsStr::new("ext"),
                OsStr::new("fsm"),
                OsStr::new("sqm"),
                OsStr::new("hpp"),
                OsStr::new("cfg"),
                OsStr::new("sqs"),
                OsStr::new("h"),
                OsStr::new("sqfc"),
            ]
            .contains(&ext),
        }
    }

    #[must_use]
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
