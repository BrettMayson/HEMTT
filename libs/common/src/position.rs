use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Line and column of a token
pub struct LineCol(pub usize, pub (usize, usize));

impl LineCol {
    #[cfg(feature = "lsp")]
    #[must_use]
    /// Convert to an LSP [`lsp_types::Position`]
    pub fn to_lsp(&self) -> lsp_types::Position {
        #[allow(clippy::cast_possible_truncation)]
        lsp_types::Position::new(self.1 .0 as u32 - 1, self.1 .1 as u32 - 1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Position of a token in a source file
pub struct Position {
    start: LineCol,
    end: LineCol,
    path: Option<PathBuf>,
}

impl Position {
    #[must_use]
    /// Create a new position
    pub const fn new(start: LineCol, end: LineCol, path: PathBuf) -> Self {
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
            start: LineCol(0, (0, 0)),
            end: LineCol(0, (0, 0)),
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
    pub const fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    #[must_use]
    /// Get the path of the source file or "%builtin%" if there is no path
    pub fn path_or_builtin(&self) -> String {
        self.path.as_ref().map_or_else(
            || String::from("%builtin%"),
            |p| p.display().to_string().replace('\\', "/"),
        )
    }

    #[cfg(feature = "lsp")]
    #[must_use]
    /// Convert to an LSP [`lsp_types::Range`]
    pub fn to_lsp(&self) -> Range {
        lsp_types::Range::new(self.start.to_lsp(), self.end.to_lsp())
    }
}
