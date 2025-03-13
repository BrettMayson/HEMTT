#[derive(thiserror::Error, Debug)]
/// Error type for the PBO writer/reader
pub enum Error {
    #[error("IO Error: {0}")]
    /// [`std::io::Error`]
    Io(#[from] std::io::Error),

    #[error("Unsupported file type: {0}")]
    /// Unsupported file type
    UnsupportedFileType(String),
    #[error("Invalid compression value: {0}")]
    /// Invalid compression value
    InvalidCompressionValue(u32),

    #[error("WAV Error: {0}")]
    /// Error while reading or writing a WAV file
    Wav(#[from] hound::Error),
    #[error("Ogg Error: {0}")]
    /// Error while reading an OGG file
    Ogg(#[from] vorbis_rs::VorbisError),
    #[error("MP3 Error: {0}")]
    /// Error while reading an MP3 file
    Mp3(#[from] puremp3::Error),
}
