mod hover;
mod semantic;

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use dashmap::DashMap;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::reporting::Token;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::SemanticToken;
use tracing::{error, warn};
use url::Url;

use crate::{
    files::FileCache,
    workspace::{EditorWorkspace, EditorWorkspaces},
    TextDocumentItem,
};

#[derive(Clone)]
pub struct SqfAnalyzer {
    tokens: Arc<DashMap<Url, Vec<Arc<Token>>>>,
    semantic: Arc<RwLock<HashMap<Url, Vec<SemanticToken>>>>,
    databases: Arc<DashMap<EditorWorkspace, Database>>,
}

impl SqfAnalyzer {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<SqfAnalyzer> = LazyLock::new(|| SqfAnalyzer {
            tokens: Arc::new(DashMap::new()),
            semantic: Arc::new(RwLock::new(HashMap::new())),
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
        let database = {
            if !self.databases.contains_key(&workspace) {
                let database = match Database::a3_with_workspace(workspace.root(), false) {
                    Ok(database) => database,
                    Err(e) => {
                        error!("Failed to create database: {:?}", e);
                        Database::a3(false)
                    }
                };
                self.databases.insert(workspace.clone(), database);
            }
            self.databases.get(&workspace).unwrap()
        };

        let text = FileCache::get().text(&document.uri).unwrap();

        self.process_semantic_tokens(document.uri.clone(), text, source, &database)
            .await;
    }
}
