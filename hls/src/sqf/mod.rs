mod hover;
mod semantic;

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use dashmap::DashMap;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::reporting::Token;
use ropey::Rope;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::SemanticToken;
use tracing::{error, warn};
use url::Url;

use crate::{
    workspace::{EditorWorkspace, EditorWorkspaces},
    TextDocumentItem, TextInformation,
};

#[derive(Clone)]
pub struct SqfAnalyzer {
    ropes: Arc<RwLock<HashMap<Url, Rope>>>,
    tokens: Arc<DashMap<Url, Vec<Arc<Token>>>>,
    semantic: Arc<RwLock<HashMap<Url, Vec<SemanticToken>>>>,
    databases: Arc<DashMap<EditorWorkspace, Database>>,
}

impl SqfAnalyzer {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<SqfAnalyzer> = LazyLock::new(|| SqfAnalyzer {
            ropes: Arc::new(RwLock::new(HashMap::new())),
            tokens: Arc::new(DashMap::new()),
            semantic: Arc::new(RwLock::new(HashMap::new())),
            databases: Arc::new(DashMap::new()),
        });
        (*SINGLETON).clone()
    }

    pub async fn on_change<'a>(&self, document: TextDocumentItem<'a>) {
        let url = document.uri;
        if !url.path().ends_with(".sqf") {
            return;
        }
        let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(&url).await else {
            warn!("Failed to find workspace for {:?}", url);
            return;
        };
        let source = if let Ok(source) = workspace.join_url(&url) {
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

        match document.text {
            TextInformation::Full(text) => {
                let mut ropes = self.ropes.write().await;
                ropes.insert(url.clone(), Rope::from_str(text));
            }
            TextInformation::Changes(changes) => {
                let mut ropes = self.ropes.write().await;
                let rope = ropes
                    .entry(url.clone())
                    .or_insert_with(|| Rope::from_str(""));
                for change in changes {
                    if let Some(range) = change.range {
                        let start = rope.line_to_char(range.start.line as usize)
                            + range.start.character as usize;
                        rope.remove(
                            start
                                ..rope.line_to_char(range.end.line as usize)
                                    + range.end.character as usize,
                        );
                        rope.insert(start, change.text.as_str());
                    }
                }
            }
        }
        let text = self.ropes.read().await.get(&url).unwrap().to_string();

        self.process_semantic_tokens(url, text, source, &database)
            .await;
    }
}
