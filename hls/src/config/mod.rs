use std::sync::LazyLock;

use tower_lsp::Client;
use url::Url;

use crate::workspace::EditorWorkspace;

mod lints;

#[derive(Clone)]
pub struct ConfigAnalyzer {}

impl ConfigAnalyzer {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<ConfigAnalyzer> = LazyLock::new(|| ConfigAnalyzer {});
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
