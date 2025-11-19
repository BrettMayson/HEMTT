use hemtt_workspace::position::LineCol;
use tower_lsp::lsp_types::Position;

pub trait ToPosition {
    #[allow(dead_code)]
    fn to_position(&self) -> Position;
}

impl ToPosition for LineCol {
    fn to_position(&self) -> Position {
        Position::new(
            u32::try_from(self.1.0).expect("Failed to convert line to u32"),
            u32::try_from(self.1.1).expect("Failed to convert column to u32"),
        )
    }
}
