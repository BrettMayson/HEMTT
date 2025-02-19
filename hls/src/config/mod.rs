use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use hemtt_workspace::{reporting::Processed, WorkspacePath};
use tower_lsp::Client;
use url::Url;

use crate::workspace::EditorWorkspace;

mod goto;
mod lints;
mod signature;

#[derive(Clone)]
pub struct ConfigAnalyzer {
    processed: Arc<DashMap<WorkspacePath, Processed>>,
}

impl ConfigAnalyzer {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<ConfigAnalyzer> = LazyLock::new(|| ConfigAnalyzer {
            processed: Arc::new(DashMap::new()),
        });
        (*SINGLETON).clone()
    }

    pub async fn workspace_added(&self, workspace: EditorWorkspace, client: Client) {
        lints::workspace_added(workspace, client).await;
    }

    pub async fn did_save(&self, url: Url, client: Client) {
        lints::did_save(url, client).await;
    }

    pub fn save_processed(&self, source: WorkspacePath, processed: Processed) {
        self.processed.insert(source, processed);
    }
}
