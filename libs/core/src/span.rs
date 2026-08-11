use crate::store::FileId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Represents a span of text in a source file, defined by a start and end byte offset.
pub struct Span {
    file: FileId,
    start: usize,
    end: usize,
}

impl Span {
    #[must_use]
    /// Creates a new `Span` with the given file ID, start, and end offsets.
    pub const fn new(file: FileId, start: usize, end: usize) -> Self {
        Self { file, start, end }
    }

    #[must_use]
    /// Returns the file ID associated with this span.
    pub const fn file(&self) -> FileId {
        self.file
    }

    #[must_use]
    /// Returns the start offset of the span.
    pub const fn start(&self) -> usize {
        self.start
    }

    #[must_use]
    /// Returns the end offset of the span.
    pub const fn end(&self) -> usize {
        self.end
    }

    #[must_use]
    /// Returns the range of the span as a `std::ops::Range<usize>`.
    pub const fn as_range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }

    #[must_use]
    /// Is the span empty? A span is considered empty if its start offset is greater than or equal to its end offset.
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_is_empty() {
        let span = Span::new(1, 5, 5);
        assert!(span.is_empty());

        let span = Span::new(1, 5, 4);
        assert!(span.is_empty());

        let span = Span::new(1, 5, 6);
        assert!(!span.is_empty());
    }
}
