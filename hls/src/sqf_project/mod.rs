use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, RwLock},
};

use hemtt_preprocessor::Processor;
use hemtt_sqf::{parser::database::Database, Statements};
use hemtt_workspace::{reporting::Processed, WorkspacePath};
use tracing::{debug, error, warn};
use url::Url;

use crate::workspace::EditorWorkspaces;

#[allow(dead_code)]
pub struct CacheBundle {
    pub processed: Processed,
    pub source: WorkspacePath,
    pub statements: Statements,
    pub database: Database,
}

#[derive(Clone)]
pub struct SqfCache {
    files: Arc<RwLock<HashMap<Url, CacheBundle>>>,
}

impl SqfCache {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<SqfCache> = LazyLock::new(|| SqfCache {
            files: Arc::new(RwLock::new(HashMap::new())),
        });
        (*SINGLETON).clone()
    }

    pub async fn cache(url: Url) {
        if !url.path().ends_with(".sqf") {
            return;
        }
        let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(&url).await else {
            warn!("Failed to find workspace for {:?}", url);
            return;
        };
        let Ok(source) = workspace.join_url(&url) else {
            warn!("Failed to join workspace and file");
            return;
        };
        debug!("Caching {:?}", source);
        let processed = match Processor::run(&source) {
            Ok(processed) => processed,
            Err(e) => {
                warn!(
                    "Failed to process {} {:?}",
                    url,
                    e.1.get_code().unwrap().diagnostic()
                );
                return;
            }
        };
        let database = match Database::a3_with_workspace(workspace.root(), false) {
            Ok(database) => database,
            Err(e) => {
                error!("Failed to create database {:?}", e);
                return;
            }
        };
        match hemtt_sqf::parser::run(&database, &processed) {
            Ok(sqf) => {
                Self::get().files.write().unwrap().insert(
                    url,
                    CacheBundle {
                        processed,
                        source,
                        statements: sqf,
                        database,
                    },
                );
            }
            Err(e) => {
                warn!("Failed to parse {}: {e:?}", url);
            }
        }
    }
}
