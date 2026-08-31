#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ParserError(#[from] crate::parser::ParserError),
    #[cfg(feature = "compiler")]
    #[error(transparent)]
    CompileError(#[from] crate::compiler::CompileError),
    #[cfg(feature = "compiler")]
    #[error(transparent)]
    SerializeError(#[from] crate::compiler::serializer::SerializeError),
    #[error("Custom command error: {0}")]
    CustomCommandError(String),
    #[error("Custom command could not be read: {0}")]
    CustomCommandIo(String),
}
