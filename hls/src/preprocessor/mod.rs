mod goto;
mod signature;

use std::{
    collections::HashSet,
    sync::{Arc, LazyLock},
};

use dashmap::DashMap;
use hemtt_workspace::{
    WorkspacePath,
    reporting::{CacheProcessed, Processed},
};
use tokio::sync::Mutex;
use url::Url;

use crate::workspace::EditorWorkspaces;

#[derive(Clone)]
pub struct PreprocessorAnalyzer {
    processed: Arc<DashMap<WorkspacePath, CacheProcessed>>,
    in_progress: Arc<Mutex<HashSet<WorkspacePath>>>,
}

impl PreprocessorAnalyzer {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<PreprocessorAnalyzer> = LazyLock::new(|| PreprocessorAnalyzer {
            processed: Arc::new(DashMap::new()),
            in_progress: Arc::new(Mutex::new(HashSet::new())),
        });
        (*SINGLETON).clone()
    }

    pub async fn is_in_progress(&self, source: &WorkspacePath) -> bool {
        self.in_progress.lock().await.contains(source)
    }

    pub async fn mark_in_progress(&self, source: WorkspacePath) {
        self.in_progress.lock().await.insert(source);
    }

    pub async fn mark_done(&self, source: WorkspacePath) {
        self.in_progress.lock().await.remove(&source);
    }

    pub async fn wait_until_done(&self, source: WorkspacePath) {
        while self.is_in_progress(&source).await {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
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
        // Wait for the save job to start
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
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
        self.wait_until_done(source.clone()).await;
        let Some(cache) = self.processed.get(&source) else {
            tracing::warn!("Failed to find cache for {:?}", source);
            return None;
        };
        Some(cache.output.clone())
    }
}
