mod hover;
mod locate;

use std::{collections::HashMap, mem::MaybeUninit, rc::Rc, sync::RwLock};

use hemtt_preprocessor::Processor;
use hemtt_sqf::{parser::database::Database, Statements};
use hemtt_workspace::{reporting::Processed, WorkspacePath};
use tracing::{debug, error, warn};
use url::Url;

use crate::workspace::EditorWorkspaces;

#[derive(Clone)]
pub struct SqfCache {
    files: Rc<RwLock<HashMap<Url, (Processed, WorkspacePath, Statements, Database)>>>,
}

impl SqfCache {
    pub fn get() -> Self {
        static mut SINGLETON: MaybeUninit<SqfCache> = MaybeUninit::uninit();
        static mut INIT: bool = false;
        unsafe {
            if !INIT {
                SINGLETON = MaybeUninit::new(Self {
                    files: Rc::new(RwLock::new(HashMap::new())),
                });
                INIT = true;
            }
            SINGLETON.assume_init_ref().clone()
        }
    }

    pub fn cache(file: Url) {
        let workspace = if let Some(workspace) = EditorWorkspaces::get().guess_workspace(&file) {
            workspace
        } else {
            // TODO: create a new workspace just for this file
            warn!("No workspace found for {}", file);
            return;
        };
        let Ok(source) = workspace.join_url(&file) else {
            warn!("Failed to join workspace and file");
            return;
        };
        debug!("Caching {:?}", source);
        let processed = match Processor::run(&source) {
            Ok(processed) => processed,
            Err(e) => {
                warn!(
                    "Failed to process {} {:?}",
                    file,
                    e.get_code().unwrap().diagnostic()
                );
                return;
            }
        };
        let database = match Database::a3_with_workspace(workspace.root()) {
            Ok(database) => database,
            Err(e) => {
                error!("Failed to create database {:?}", e);
                return;
            }
        };
        match hemtt_sqf::parser::run(&database, &processed) {
            Ok(sqf) => {
                Self::get()
                    .files
                    .write()
                    .unwrap()
                    .insert(file, (processed, source, sqf, database));
            }
            Err(e) => {
                warn!("Failed to parse {}: {e:?}", file);
            }
        }
    }
}
