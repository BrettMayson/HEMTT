use serde::{Deserialize, Deserializer, Serialize};

#[derive(Default, PartialEq, Eq, Debug, Copy, Clone)]
pub enum PDriveOption {
    Disallow,
    #[default]
    Ignore,
    Require,
}

impl<'de> Deserialize<'de> for PDriveOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "disallow" => Ok(Self::Disallow),
            "require" => Ok(Self::Require),
            "ignore" => Ok(Self::Ignore),
            _ => Err(serde::de::Error::custom(
                "valid values are disallow, ignore, require",
            )),
        }
    }
}

impl Serialize for PDriveOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Disallow => serializer.serialize_str("disallow"),
            Self::Ignore => serializer.serialize_str("ignore"),
            Self::Require => serializer.serialize_str("required"),
        }
    }
}
