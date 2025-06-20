mod compiled;
mod hover;
mod lints;

use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::reporting::Token;
use tower_lsp::Client;
use tracing::error;
use url::Url;

use crate::{TextDocumentItem, workspace::EditorWorkspace};

#[derive(Clone)]
pub struct SqfAnalyzer {
    tokens: Arc<DashMap<Url, Vec<Arc<Token>>>>,
    databases: Arc<DashMap<EditorWorkspace, Arc<Database>>>,
}

impl SqfAnalyzer {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<SqfAnalyzer> = LazyLock::new(|| SqfAnalyzer {
            tokens: Arc::new(DashMap::new()),
            databases: Arc::new(DashMap::new()),
        });
        (*SINGLETON).clone()
    }

    pub async fn on_change(&self, document: &TextDocumentItem<'_>) {
        if !document.uri.path().ends_with(".sqf") {
            return;
        }
    }

    pub async fn workspace_added(&self, workspace: EditorWorkspace, client: Client) {
        self.check_lints(workspace, client).await;
    }

    pub async fn on_open(&self, url: Url, client: Client) {
        self.partial_recheck_lints(url, client).await;
    }

    pub async fn on_save(&self, url: Url, client: Client) {
        self.partial_recheck_lints(url, client).await;
    }

    pub async fn on_close(&self, url: &Url) {
        self.tokens.remove(url);
    }

    async fn get_database(&self, workspace: &EditorWorkspace) -> Arc<Database> {
        if !self.databases.contains_key(workspace) {
            let database = match Database::a3_with_workspace(workspace.root(), false) {
                Ok(database) => database,
                Err(e) => {
                    error!("Failed to create database: {:?}", e);
                    Database::a3(false)
                }
            };
            self.databases.insert(workspace.clone(), Arc::new(database));
        }
        self.databases.get(workspace).unwrap().clone()
    }
}
