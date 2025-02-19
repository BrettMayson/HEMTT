use tower_lsp::lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location};
use tracing::warn;

use crate::{common::get_definition, workspace::EditorWorkspaces};

use super::SqfAnalyzer;

impl SqfAnalyzer {
    pub async fn goto_definition(
        &self,
        params: &GotoDefinitionParams,
    ) -> Option<GotoDefinitionResponse> {
        let url = &params.text_document_position_params.text_document.uri;
        let path = url.to_file_path().ok()?;
        if !matches!(path.extension().and_then(|ext| ext.to_str()), Some("sqf")) {
            return None;
        }
        let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(url).await else {
            warn!("Failed to find workspace for {:?}", url);
            return None;
        };
        let source = workspace.join_url(url).ok()?;
        let processed = self.processed.get(&source)?;
        let definition = get_definition(
            &source,
            &params.text_document_position_params.position,
            &processed,
        )
        .await;
        definition.map(|def| {
            GotoDefinitionResponse::Scalar(Location {
                uri: workspace.to_url(def.0.path()),
                range: def.0.to_lsp(),
            })
        })
    }
}
