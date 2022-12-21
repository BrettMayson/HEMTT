use hemtt_error::thiserror;

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Expected Major")]
    ExpectedMajor,

    #[error("Expected Minor")]
    ExpectedMinor,

    #[error("Expected Patch")]
    ExpectedPatch,

    #[error("Expected Build")]
    ExpectedBuild,

    #[error("Not a valid component: {0}")]
    InvalidComponent(String),
}
