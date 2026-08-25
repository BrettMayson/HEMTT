use hemtt_workspace::position::LineCol;
use tower_lsp::lsp_types::Position;

pub trait ToPosition {
    #[allow(dead_code)]
    fn to_position(&self) -> Position;
}

impl ToPosition for LineCol {
    fn to_position(&self) -> Position {
        Position::new(
            u32::try_from(self.1.0).unwrap_or(u32::MAX),
            u32::try_from(self.1.1).unwrap_or(u32::MAX),
        )
    }
}

#[cfg(test)]
mod tests {
    use hemtt_workspace::position::LineCol;

    use super::ToPosition;

    #[test]
    fn saturates_instead_of_panicking() {
        let position = LineCol(0, (usize::MAX, usize::MAX)).to_position();
        assert_eq!(position.line, u32::MAX);
        assert_eq!(position.character, u32::MAX);
    }
}
