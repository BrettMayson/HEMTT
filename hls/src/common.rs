use hemtt_workspace::{reporting::Processed, WorkspacePath};
use tower_lsp::lsp_types::Position;

pub async fn get_definition<'a>(
    source: &WorkspacePath,
    position: &Position,
    processed: &'a Processed,
) -> Option<(
    &'a hemtt_workspace::position::Position,
    &'a Vec<hemtt_workspace::position::Position>,
)> {
    processed.usage().iter().find(|def| {
        def.1.iter().any(|usage| {
            if usage.path().as_str() != source.as_str() {
                return false;
            }
            usage.start().1 .0 - 1 <= position.line as usize
                && usage.end().1 .0 > position.line as usize
                && usage.start().1 .1 - 1 <= position.character as usize
                && usage.end().1 .1 > position.character as usize
        })
    })
}
