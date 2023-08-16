use hemtt_common::error::thiserror;

#[derive(thiserror::Error, Debug)]
/// Error type for the signing module
pub enum Error {
    #[error("RSA Error: {0}")]
    /// [rsa::errors::Error]
    Rsa(#[from] rsa::errors::Error),
    #[error("IO Error: {0}")]
    /// [std::io::Error]
    Io(Box<std::io::Error>),
    #[error("PBO Error: {0}")]
    /// [hemtt_pbo::Error]
    Pbo(Box<hemtt_pbo::Error>),

    #[error("Invalid length while reading")]
    /// Invalid length while reading a file
    InvalidLength,

    #[error("Missing authority")]
    /// No authority was provided
    MissingAuthority,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(Box::new(e))
    }
}

impl From<hemtt_pbo::Error> for Error {
    fn from(e: hemtt_pbo::Error) -> Self {
        Self::Pbo(Box::new(e))
    }
}
