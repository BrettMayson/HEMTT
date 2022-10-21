pub type LineCol = (usize, (usize, usize));

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Position {
    start: LineCol,
    end: LineCol,
    path: String,
}

impl Position {
    pub const fn new(start: LineCol, end: LineCol, source: String) -> Self {
        Self {
            start,
            end,
            path: source,
        }
    }

    pub fn builtin() -> Self {
        Self {
            start: (0, (0, 0)),
            end: (0, (0, 0)),
            path: String::from("%builtin%"),
        }
    }

    pub const fn start(&self) -> &LineCol {
        &self.start
    }

    pub const fn end(&self) -> &LineCol {
        &self.end
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
