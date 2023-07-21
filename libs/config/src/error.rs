use hemtt_error::thiserror;

#[derive(thiserror::Error, Debug)]
/// Error type for the signing module
pub enum Error {
    #[error("The config file is invalid: {0}")]
    /// An ArmA config file is invalid
    ConfigInvalid(String),
}
