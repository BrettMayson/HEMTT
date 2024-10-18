use hemtt_common::error::thiserror;

#[derive(thiserror::Error, Debug)]
/// Error type for the PBO writer/reader
pub enum Error {
    #[error("IO Error: {0}")]
    /// [`std::io::Error`]
    Io(#[from] std::io::Error),

    #[error("Unsupported p3d type: {0}")]
    /// Unsupported p3d type
    UnsupportedP3DType(String),
    #[error("Unsupported lod type: {0}")]
    /// Unsupported lod type
    UnsupportedLODType(String),

    #[error("Unexpected bytes where `TAGG` expected: {0}")]
    /// Unexpected bytes where `TAGG` expected
    UnexpectedBytesTagg(String),

    #[error("Exceeded max lod count: {0}, max: 4294967295")]
    /// Exceeded max lod count
    ExceededMaxLodCount(u64),
    #[error("Exceeded max vertex count: {0}, max: 4294967295")]
    /// Exceeded max vertex count
    ExceededMaxVertexCount(u64),
    #[error("Exceeded max face count: {0}, max: 4294967295")]
    /// Exceeded max face count
    ExceededMaxFaceCount(u64),
    #[error("Exceeded max face normal count: {0}, max: 4294967295")]
    /// Exceeded max face normal count
    ExceededMaxFaceNormalCount(u64),
    #[error("Exceeded max point count: {0}, max: 4294967295")]
    /// Exceeded max point count
    ExceededMaxPointCount(u64),
    #[error("Exceeded tagg length: {0}, max: 4294967295")]
    /// Exceeded tagg length
    ExceededTaggLength(u64),

    #[error("Invalid face vertex count: {0}")]
    /// Invalid face vertex count
    InvalidFaceVertexCount(u32),
}
