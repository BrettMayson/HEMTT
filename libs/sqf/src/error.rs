use hemtt_common::error::thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "asc")]
    #[error(transparent)]
    AscError(#[from] crate::asc::Error),
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
}
