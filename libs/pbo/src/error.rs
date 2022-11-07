use hemtt_error::thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HEMTT does not support the encountered PBO Mime type: {0}")]
    UnsupportedMime(u32),
    #[error("Unexpected data after PBO checksum")]
    UnexpectedDataAfterChecksum,
    #[error("File is too large for PBO format")]
    FileTooLarge,
    #[error("Invalid prefix: {0}")]
    InvalidPrefix(String),
}
