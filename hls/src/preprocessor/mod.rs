mod goto;
mod signature;

use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use hemtt_workspace::{
    reporting::{CacheProcessed, Processed},
    WorkspacePath,
};
use url::Url;

use crate::workspace::EditorWorkspaces;

#[derive(Clone)]
pub struct PreprocessorAnalyzer {
    processed: Arc<DashMap<WorkspacePath, CacheProcessed>>,
}

impl PreprocessorAnalyzer {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<PreprocessorAnalyzer> = LazyLock::new(|| PreprocessorAnalyzer {
            processed: Arc::new(DashMap::new()),
        });
        (*SINGLETON).clone()
    }

    pub fn save_processed(&self, source: WorkspacePath, processed: Processed) {
        self.processed.insert(source, processed.cache());
    }

    pub async fn on_close(&self, url: &Url) {
        let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(url).await else {
            tracing::warn!("Failed to find workspace for {:?}", url);
            return;
        };
        let Ok(source) = workspace.join_url(url) else {
            tracing::warn!("Failed to join url {:?}", url);
            return;
        };
        if source.extension() == Some("sqf".to_string()) && self.processed.remove(&source).is_some()
        {
            tracing::debug!("sqf: removed processed cache for {}", source);
        }
    }

    pub async fn get_processed(&self, url: Url) -> Option<String> {
        let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(&url).await else {
            tracing::warn!("Failed to find workspace for {:?}", url);
            return None;
        };
        let Ok(mut source) = workspace.join_url(&url) else {
            tracing::warn!("Failed to join url {:?}", url);
            return None;
        };
        if source.extension() == Some("cpp".to_string()) {
            source = source.parent();
        }
        let Some(cache) = self.processed.get(&source) else {
            tracing::warn!("Failed to find cache for {:?}", source);
            return None;
        };
        Some(cache.output.clone())
    }
}
