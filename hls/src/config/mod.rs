use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use hemtt_workspace::addons::DefinedFunctions;
use tower_lsp::Client;
use url::Url;

use crate::workspace::EditorWorkspace;

mod lints;

#[derive(Clone)]
pub struct ConfigAnalyzer {
    pub(crate) functions_defined: Arc<DashMap<String, DefinedFunctions>>,
}

impl ConfigAnalyzer {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<ConfigAnalyzer> = LazyLock::new(|| ConfigAnalyzer {
            functions_defined: Arc::new(DashMap::new()),
        });
        (*SINGLETON).clone()
    }

    pub async fn workspace_added(&self, workspace: EditorWorkspace, client: Client) {
        lints::workspace_added(workspace, client).await;
    }

    pub async fn on_open(&self, url: Url, client: Client) {
        lints::process(url, client).await;
    }

    pub async fn on_save(&self, url: Url, client: Client) {
        lints::process(url, client).await;
    }
}
