use vfs::VfsPath;

/// Line and column of a token
pub type LineCol = (usize, (usize, usize));

#[derive(Clone, Debug, PartialEq, Eq)]
/// Position of a token in a source file
pub struct Position {
    start: LineCol,
    end: LineCol,
    path: Option<VfsPath>,
}

impl Position {
    #[must_use]
    /// Create a new position
    pub const fn new(start: LineCol, end: LineCol, path: VfsPath) -> Self {
        Self {
            start,
            end,
            path: Some(path),
        }
    }

    #[must_use]
    /// Create a new position for a built-in token
    pub const fn builtin() -> Self {
        Self {
            start: (0, (0, 0)),
            end: (0, (0, 0)),
            path: None,
        }
    }

    #[must_use]
    /// Get the start [`LineCol`] of the token
    pub const fn start(&self) -> &LineCol {
        &self.start
    }

    #[must_use]
    /// Get the end [`LineCol`] of the token
    pub const fn end(&self) -> &LineCol {
        &self.end
    }

    #[must_use]
    /// Get the path of the source file
    pub const fn path(&self) -> Option<&VfsPath> {
        self.path.as_ref()
    }

    #[must_use]
    pub fn path_or_builtin(&self) -> String {
        self.path.as_ref().map_or_else(
            || String::from("%builtin%"),
            |p| p.as_str().replace('\\', "/"),
        )
    }
}
