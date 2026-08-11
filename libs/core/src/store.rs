use std::sync::Arc;

/// A unique identifier for a source file.
pub type FileId = u32;

/// A trait that abstracts file operations for a source store.
pub trait SourceStore {
    /// Reads the contents of a file given its `FileId`.
    ///
    /// # Errors
    /// [`Error`](Error) is returned by the implementation if the file cannot be read.
    fn read(&self, id: FileId) -> Result<Arc<str>, Error>;
    /// Retrieves the path of a file given its `FileId`.
    fn path(&self, id: FileId) -> Option<Arc<str>>;
    /// Retrieves the name of the a file given its `FileId`.
    fn name(&self, id: FileId) -> Option<Arc<str>> {
        self.path(id).map(|path| {
            let path_str = path.as_ref();
            path_str
                .rfind('\\')
                .map_or_else(|| path.clone(), |pos| Arc::from(&path_str[pos + 1..]))
        })
    }
    /// Looks for a file relative to the given `FileId` or absolute from root and returns its `FileId` if found.
    ///
    /// # Errors
    /// [`Error`](Error) is returned by the implementation if the file cannot be found.
    fn find(&self, id: FileId, path: &str) -> Result<Option<FileId>, Error>;
}

/// An error type for `SourceStore` operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Source not found")]
    /// A source file was not found in the store.
    SourceNotFound(FileId),

    #[error("Incorrect separator in path")]
    /// The path provided to the store contains an incorrect separator.
    ///
    /// HEMTT uses backslashes (`\`) as the path separator, and this error indicates that a forward slash (`/`) was used instead.
    IncorrectSeparator,

    #[error("IO error: {0}")]
    /// An I/O error occurred while accessing the source store.
    Io(#[from] std::io::Error),
}
