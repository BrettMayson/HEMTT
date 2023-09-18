//! Position of a token in a source file

use std::path::PathBuf;

use crate::workspace::WorkspacePath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    #[must_use]
    /// Get the line of the token
    pub const fn line(&self) -> usize {
        self.1 .0
    }

    #[must_use]
    /// Get the column of the token
    pub const fn column(&self) -> usize {
        self.1 .1
    }

    #[must_use]
    /// Get the offset of the token
    pub const fn offset(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// Position of a token in a source file
pub struct Position {
    start: LineCol,
    end: LineCol,
    path: WorkspacePath,
}

impl Position {
    #[must_use]
    /// Create a new position
    pub const fn new(start: LineCol, end: LineCol, path: WorkspacePath) -> Self {
        Self { start, end, path }
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
    pub const fn path(&self) -> &WorkspacePath {
        &self.path
    }

    #[cfg(feature = "lsp")]
    #[must_use]
    /// Convert to an LSP [`lsp_types::Range`]
    pub fn to_lsp(&self) -> Range {
        lsp_types::Range::new(self.start.to_lsp(), self.end.to_lsp())
    }
}
