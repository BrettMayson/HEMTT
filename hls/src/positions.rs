use hemtt_workspace::position::LineCol;
use tower_lsp::lsp_types::Position;

pub trait ToPosition {
    fn to_position(&self) -> Position;
}

impl ToPosition for LineCol {
    fn to_position(&self) -> Position {
        Position::new(self.1 .0 as u32, self.1 .1 as u32)
    }
}
