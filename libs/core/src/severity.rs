#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents the severity level of a diagnostic message.
pub enum Severity {
    /// An error that prevents the program from compiling or running correctly.
    Error,
    /// A warning that indicates a potential issue or bad practice.
    Warning,
    /// A suggestion for improving the code, but not necessarily an error or warning.
    Help,
    /// A note that provides additional context or information.
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("invalid severity level: {0}")]
    InvalidSeverity(String),
}

impl TryFrom<String> for Severity {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, <Self as TryFrom<String>>::Error> {
        if value.eq_ignore_ascii_case("error") {
            Ok(Self::Error)
        } else if value.eq_ignore_ascii_case("warning") {
            Ok(Self::Warning)
        } else if value.eq_ignore_ascii_case("help") {
            Ok(Self::Help)
        } else if value.eq_ignore_ascii_case("note") {
            Ok(Self::Note)
        } else {
            Err(Error::InvalidSeverity(value))
        }
    }
}

impl TryFrom<&str> for Severity {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        Self::try_from(value.to_string())
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Help => write!(f, "help"),
            Self::Note => write!(f, "note"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_severity_try_from() {
        use super::Severity;
        assert_eq!(Severity::try_from("error"), Ok(Severity::Error));
        assert_eq!(Severity::try_from("warning"), Ok(Severity::Warning));
        assert_eq!(Severity::try_from("help"), Ok(Severity::Help));
        assert_eq!(Severity::try_from("note"), Ok(Severity::Note));
        assert!(Severity::try_from("invalid").is_err());
    }

    #[test]
    fn test_severity_try_from_case_insensitive() {
        use super::Severity;
        assert_eq!(Severity::try_from("ERROR"), Ok(Severity::Error));
        assert_eq!(Severity::try_from("Warning"), Ok(Severity::Warning));
        assert_eq!(Severity::try_from("HeLp"), Ok(Severity::Help));
        assert_eq!(Severity::try_from("NoTe"), Ok(Severity::Note));
    }

    #[test]
    fn test_severity_display() {
        use super::Severity;
        for severity in ["Error", "Warning", "Help", "Note"] {
            let severity_enum =
                Severity::try_from(severity).expect("Failed to convert string to Severity");
            assert_eq!(severity_enum.to_string(), severity.to_lowercase());
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::Severity;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for Severity {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.to_string())
        }
    }

    impl<'de> Deserialize<'de> for Severity {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Self::try_from(s).map_err(serde::de::Error::custom)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_serialize_deserialize() {
            let severities = [
                Severity::Error,
                Severity::Warning,
                Severity::Help,
                Severity::Note,
            ];
            for severity in &severities {
                let serialized =
                    serde_json::to_string(severity).expect("Failed to serialize Severity");
                let deserialized: Severity =
                    serde_json::from_str(&serialized).expect("Failed to deserialize Severity");
                assert_eq!(*severity, deserialized);
            }
        }
    }
}
