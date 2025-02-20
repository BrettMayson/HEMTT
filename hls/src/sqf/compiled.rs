use hemtt_preprocessor::Processor;
use url::Url;

use crate::workspace::EditorWorkspaces;

use super::SqfAnalyzer;

impl SqfAnalyzer {
    pub async fn get_compiled(&self, url: Url) -> Option<String> {
        if !url.path().ends_with(".sqf") {
            return None;
        }
        let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(&url).await else {
            tracing::warn!("Failed to find workspace for {:?}", url);
            return None;
        };
        let source = workspace.join_url(&url).ok()?;
        let database = self.get_database(&workspace).await;
        match Processor::run(&source) {
            Ok(processed) => match hemtt_sqf::parser::run(&database, &processed) {
                Ok(sqf) => {
                    let Ok(compiled) = sqf.optimize().compile(&processed) else {
                        tracing::error!("Failed to compile SQF");
                        return None;
                    };
                    Some(compiled.display().to_string())
                }
                Err(e) => {
                    tracing::error!("Failed to parse SQF: {:?}", e);
                    None
                }
            },
            Err(e) => {
                tracing::error!("Failed to preprocess SQF: {:?}", e);
                None
            }
        }
    }
}
