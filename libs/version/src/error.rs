use hemtt_error::thiserror;

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
}
