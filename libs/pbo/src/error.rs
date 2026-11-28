#[derive(thiserror::Error, Debug)]
/// Error type for the PBO writer/reader
pub enum Error {
    #[error("IO Error: {0}")]
    /// [`std::io::Error`]
    Io(#[from] std::io::Error),

    #[error("HEMTT does not support the encountered PBO Mime type: {0}")]
    /// HEMTT does not support the encountered PBO Mime type
    UnsupportedMime(u32),
    #[error("PBO contains a name that is not valid UTF-8: `{0}`")]
    /// A file name or property in the PBO is not valid UTF-8,
    /// usually because it was packed with a Windows code page
    InvalidName(String),
    #[error("Unexpected data after PBO checksum")]
    /// Unexpected data after PBO checksum
    UnexpectedDataAfterChecksum,
    #[error("File is too large for PBO format")]
    /// File is too large for PBO format
    FileTooLarge,
    #[error("Failed to decompress `{file}` in PBO: {reason}")]
    /// A compressed file in the PBO could not be decompressed
    Decompress {
        /// The file in the PBO
        file: String,
        /// Why it could not be decompressed
        reason: &'static str,
    },
    #[error("HEMTT does not support signing PBOs with no files")]
    /// HEMTT does not support signing PBOs with no files
    NoFiles,
}
