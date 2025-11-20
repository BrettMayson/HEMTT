mod compiled;
mod hover;
mod lints;

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use dashmap::DashMap;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::{addons::DefinedFunctions, reporting::Token};
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
    pub(crate) functions_defined: Arc<DashMap<String, HashMap<String, DefinedFunctions>>>,
}

impl SqfAnalyzer {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<SqfAnalyzer> = LazyLock::new(|| SqfAnalyzer {
            tokens: Arc::new(DashMap::new()),
            databases: Arc::new(DashMap::new()),
            functions_defined: Arc::new(DashMap::new()),
        });
        (*SINGLETON).clone()
    }

    pub async fn on_change(&self, document: &TextDocumentItem<'_>) {
        if !std::path::Path::new(document.uri.path())
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sqf"))
        {
            return;
        }
        let Some(workspace) = EditorWorkspaces::get()
            .guess_workspace_retry(&document.uri)
            .await
        else {
            warn!("Failed to find workspace for {:?}", document.uri);
            return;
        };
        let source = workspace.join_url(&document.uri).unwrap_or_else(|_| {
            hemtt_workspace::Workspace::builder()
                .memory()
                .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
                .expect("Failed to create in-memory workspace")
        });
        let text = FileCache::get().text(&document.uri).unwrap_or_default();
        let Ok(tokens) = hemtt_preprocessor::parse::str(&text, &source) else {
            warn!("Failed to parse file");
            return;
        };
        self.tokens.insert(document.uri.clone(), tokens);
    }

    pub fn workspace_added(&self, workspace: &EditorWorkspace, client: Client) {
        self.check_lints(workspace, client);
    }

    pub async fn on_open(&self, url: Url, client: Client) {
        self.partial_recheck_lints(url, client).await;
    }

    pub async fn on_save(&self, url: Url, client: Client) {
        self.partial_recheck_lints(url, client).await;
    }

    pub fn on_close(&self, url: &Url) {
        self.tokens.remove(url);
    }

    fn get_database(&self, workspace: &EditorWorkspace) -> Arc<Database> {
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
        self.databases
            .get(workspace)
            .expect("Database not found")
            .clone()
    }
}
