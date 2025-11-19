use hemtt_workspace::{WorkspacePath, reporting::CacheProcessed};
use tower_lsp::lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location, Position};
use tracing::warn;

use crate::workspace::EditorWorkspaces;

use super::PreprocessorAnalyzer;

#[derive(Debug, PartialEq, Eq)]
enum Kind {
    Config,
    Sqf,
}

impl PreprocessorAnalyzer {
    #[allow(clippy::significant_drop_tightening)]
    pub async fn goto_definition(
        &self,
        params: &GotoDefinitionParams,
    ) -> Option<GotoDefinitionResponse> {
        let url = &params.text_document_position_params.text_document.uri;
        let path = url.to_file_path().ok()?;
        let kind = match path.extension().and_then(|ext| ext.to_str()) {
            Some("hpp" | "cpp" | "ext") => Kind::Config,
            Some("sqf") => Kind::Sqf,
            _ => return None,
        };
        let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(url).await else {
            warn!("Failed to find workspace for {:?}", url);
            return None;
        };
        let source = workspace.join_url(url).ok()?;
        #[allow(clippy::redundant_clone)]
        let processed: dashmap::mapref::one::Ref<'_, WorkspacePath, CacheProcessed> =
            self.processed.get(&if kind == Kind::Config {
                source.parent()
            } else {
                source.clone()
            })?;
        let definition = get_definition(
            &source,
            params.text_document_position_params.position,
            &processed,
        );
        definition.map(|def| {
            GotoDefinitionResponse::Scalar(Location {
                uri: workspace.to_url(def.0.path()),
                range: def.0.to_lsp(),
            })
        })
    }
}

pub fn get_definition<'a>(
    source: &WorkspacePath,
    position: Position,
    processed: &'a CacheProcessed,
) -> Option<(
    &'a hemtt_workspace::position::Position,
    &'a Vec<hemtt_workspace::position::Position>,
)> {
    processed.usage.iter().find(|def| {
        def.1.iter().any(|usage| {
            if usage.path().as_str() != source.as_str() {
                return false;
            }
            usage.start().1.0 - 1 <= position.line as usize
                && usage.end().1.0 > position.line as usize
                && usage.start().1.1 <= position.character as usize
                && usage.end().1.1 >= position.character as usize
        })
    })
}
