#[derive(thiserror::Error, Debug)]
/// Error type for the signing module
pub enum Error {
    #[error("RSA Error: {0}")]
    /// [`rsa::errors::Error`]
    Rsa(#[from] rsa::errors::Error),
    #[error("IO Error: {0}")]
    /// [`std::io::Error`]
    Io(Box<std::io::Error>),
    #[error("PBO Error: {0}")]
    /// [`hemtt_pbo::Error`]
    Pbo(Box<hemtt_pbo::Error>),

    #[error("Invalid length while reading")]
    /// Invalid length while reading a file
    InvalidLength,

    #[error("Invalid private key format")]
    /// Invalid private key format
    InvalidMagic,

    #[error("Missing authority")]
    /// No authority was provided
    AuthorityMissing,

    #[error("Invalid file sorting")]
    /// The files in the PBO are not sorted
    InvalidFileSorting,

    #[error("Hash mismatch")]
    /// The hash of the PBO does not match the signature
    HashMismatch {
        /// The hash in the signature
        sig: String,
        /// The real hash of the PBO
        real: String,
    },

    #[error("Authority mismatch")]
    /// The authority of the key and signature do not match
    AuthorityMismatch {
        /// The authority in the signature
        sig: String,
        /// The authority in the key
        key: String,
    },

    #[error("Unknown signature version {0}")]
    /// Unknown signature version
    UknownBISignVersion(u32),
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
