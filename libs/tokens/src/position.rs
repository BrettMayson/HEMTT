pub type LineCol = (usize, (usize, usize));

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Position {
    start: LineCol,
    end: LineCol,
    path: String,
}

impl Position {
    #[must_use]
    pub const fn new(start: LineCol, end: LineCol, path: String) -> Self {
        Self { start, end, path }
    }

    #[must_use]
    pub fn builtin() -> Self {
        Self {
            start: (0, (0, 0)),
            end: (0, (0, 0)),
            path: String::from("%builtin%"),
        }
    }

    #[must_use]
    pub const fn start(&self) -> &LineCol {
        &self.start
    }

    #[must_use]
    pub const fn end(&self) -> &LineCol {
        &self.end
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
}
