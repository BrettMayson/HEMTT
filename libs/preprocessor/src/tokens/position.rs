/// Line and column of a token
pub type LineCol = (usize, (usize, usize));

#[derive(Clone, Debug, PartialEq, Eq)]
/// Position of a token in a source file
pub struct Position {
    start: LineCol,
    end: LineCol,
    path: String,
}

impl Position {
    #[must_use]
    /// Create a new position
    pub const fn new(start: LineCol, end: LineCol, path: String) -> Self {
        Self { start, end, path }
    }

    #[must_use]
    /// Create a new position for a built-in token
    pub fn builtin() -> Self {
        Self {
            start: (0, (0, 0)),
            end: (0, (0, 0)),
            path: String::from("%builtin%"),
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
    pub fn path(&self) -> &str {
        &self.path
    }
}
