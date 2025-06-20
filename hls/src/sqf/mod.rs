mod compiled;
mod hover;
mod lints;

use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::reporting::Token;
use tower_lsp::Client;
use tracing::{error, warn};
use url::Url;

use crate::{
    TextDocumentItem,
    files::FileCache,
    workspace::{EditorWorkspace, EditorWorkspaces},
};

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
        let Some(workspace) = EditorWorkspaces::get()
            .guess_workspace_retry(&document.uri)
            .await
        else {
            warn!("Failed to find workspace for {:?}", document.uri);
            return;
        };
        let source = if let Ok(source) = workspace.join_url(&document.uri) {
            source
        } else {
            hemtt_workspace::Workspace::builder()
                .memory()
                .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
                .unwrap()
        };
        let text = FileCache::get().text(&document.uri).unwrap();
        let Ok(tokens) = hemtt_preprocessor::parse::str(&text, &source) else {
            warn!("Failed to parse file");
            return;
        };
        self.tokens.insert(document.uri.clone(), tokens.clone());
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
