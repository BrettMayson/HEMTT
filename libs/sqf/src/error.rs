use hemtt_common::error::thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("One or more files are not valid SQF")]
    InvalidSQF,
    #[error(transparent)]
    ParserError(#[from] crate::parser::ParserError),
    #[cfg(feature = "compiler")]
    #[error(transparent)]
    CompileError(#[from] crate::compiler::CompileError),
    #[cfg(feature = "compiler")]
    #[error(transparent)]
    SerializeError(#[from] crate::compiler::serializer::SerializeError),
}
