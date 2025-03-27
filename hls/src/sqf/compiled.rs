use hemtt_preprocessor::Processor;
use url::Url;

use crate::{Backend, ProviderParams, workspace::EditorWorkspaces};

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
                Ok(sqf) => match sqf.optimize().compile(&processed) {
                    Ok(compiled) => Some(compiled.display().to_string()),
                    Err(e) => {
                        tracing::error!("Failed to compile SQF: {:?}", e);
                        Some(e.to_string())
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to parse SQF: {:?}", e);
                    Some(e.to_string())
                    // needs to have ansi stripped out, should wait for the error rework
                    // let workspace_files = WorkspaceFiles::new();
                    // Some(
                    //     e.codes()
                    //         .iter()
                    //         .map(|c| {
                    //             c.diagnostic()
                    //                 .map(|d| d.to_string(&workspace_files))
                    //                 .unwrap_or_default()
                    //         })
                    //         .collect::<Vec<_>>()
                    //         .join("\n\n"),
                    // )
                }
            },
            Err((_, e)) => {
                tracing::error!("Failed to preprocess SQF: {:?}", e);
                Some(e.to_string())
            }
        }
    }
}

impl Backend {
    pub async fn sqf_compiled(
        &self,
        params: ProviderParams,
    ) -> tower_lsp::jsonrpc::Result<Option<serde_json::Value>> {
        let Some(res) = SqfAnalyzer::get().get_compiled(params.url).await else {
            return Ok(None);
        };
        Ok(Some(serde_json::to_value(res).unwrap()))
    }
}
