use std::{
    collections::HashMap,
    mem::MaybeUninit,
    sync::{atomic::AtomicBool, Arc},
};

use hemtt_preprocessor::Processor;
use hemtt_workspace::{reporting::WorkspaceFiles, WorkspacePath};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use url::Url;

use crate::{
    diag_manager::DiagManager,
    workspace::{EditorWorkspace, EditorWorkspaces},
};

pub struct CacheBundle {
    pub sources: Vec<WorkspacePath>,
}

#[derive(Clone)]
pub struct ConfigCache {
    files: Arc<RwLock<HashMap<WorkspacePath, CacheBundle>>>,
}

impl ConfigCache {
    pub fn get() -> Self {
        static mut SINGLETON: MaybeUninit<ConfigCache> = MaybeUninit::uninit();
        static mut INIT: AtomicBool = AtomicBool::new(false);
        unsafe {
            if !INIT.swap(true, std::sync::atomic::Ordering::SeqCst) {
                SINGLETON = MaybeUninit::new(Self {
                    files: Arc::new(RwLock::new(HashMap::new())),
                });
                INIT.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            SINGLETON.assume_init_ref().clone()
        }
    }
}

async fn check_addons(workspace: EditorWorkspace) {
    for config in workspace.root().addons() {
        let Ok(source) = workspace.root().join(config.as_str()) else {
            warn!("failed to join config {:?}", config);
            continue;
        };
        check_addon(source, workspace.clone()).await;
    }
}

async fn check_addon(source: WorkspacePath, workspace: EditorWorkspace) {
    let mut sources = Vec::new();
    let Some(manager) = DiagManager::get() else {
        warn!("failed to get diag manager");
        return;
    };
    manager.clear_current(&format!("config:{}", source.as_str()));
    let mut lsp_diags = HashMap::new();
    match Processor::run(&source) {
        Ok(processed) => {
            let workspace_files = WorkspaceFiles::new();
            match hemtt_config::parse(None, &processed) {
                Ok(report) => {
                    info!("parsed config for {}", source);
                    for warning in report.warnings() {
                        warn!("warning: {:?}", warning);
                        let Some(diag) = warning.diagnostic() else {
                            continue;
                        };
                        let lsp_diag = diag.to_lsp(&workspace_files);
                        for (file, diag) in lsp_diag {
                            lsp_diags.entry(file).or_insert_with(Vec::new).push(diag);
                        }
                    }
                    for error in report.errors() {
                        warn!("error: {:?}", error);
                        let Some(diag) = error.diagnostic() else {
                            continue;
                        };
                        let lsp_diag = diag.to_lsp(&workspace_files);
                        for (file, diag) in lsp_diag {
                            lsp_diags.entry(file).or_insert_with(Vec::new).push(diag);
                        }
                    }
                }
                Err(err) => {
                    warn!("failed to process config: {:?}", err);
                    for error in err {
                        warn!("error: {:?}", error);
                        let Some(diag) = error.diagnostic() else {
                            continue;
                        };
                        let lsp_diag = diag.to_lsp(&workspace_files);
                        for (file, diag) in lsp_diag {
                            lsp_diags.entry(file).or_insert_with(Vec::new).push(diag);
                        }
                    }
                }
            }
            sources = processed.sources().into_iter().map(|(p, _)| p).collect();
        }
        Err((err_sources, err)) => {
            warn!("failed to parse config: {:?}", err);
            sources = err_sources;
            debug!("failed sources: {:?}", sources);
            if let hemtt_preprocessor::Error::Code(code) = err {
                let workspace_files = WorkspaceFiles::new();
                if let Some(diag) = code.diagnostic() {
                    let lsp_diag = diag.to_lsp(&workspace_files);
                    for (file, diag) in lsp_diag {
                        lsp_diags.entry(file).or_insert_with(Vec::new).push(diag);
                    }
                };
            }
        }
    }
    for (file, diags) in lsp_diags {
        manager.set_current(
            &format!("config:{}", source.as_str()),
            &workspace.to_url(&file),
            diags,
        );
    }
    manager.sync();
    let cache = ConfigCache::get();
    if !sources.is_empty() {
        cache
            .files
            .write()
            .await
            .insert(source.clone(), CacheBundle { sources });
    } else {
        cache.files.write().await.remove(&source);
    }
}

pub async fn workspace_added(workspace: EditorWorkspace) {
    tokio::spawn(check_addons(workspace));
}

pub async fn did_save(url: Url) {
    let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(&url).await else {
        warn!("Failed to find workspace for {:?}", url);
        return;
    };
    let recheck_addons = {
        let cache = ConfigCache::get();
        let files = cache.files.read().await;
        files
            .iter()
            .filter_map(|(path, bundle)| {
                if bundle.sources.iter().any(|source| {
                    workspace
                        .join_url(&url)
                        .map(|joined| joined == *source)
                        .unwrap_or(false)
                }) {
                    info!("rechecking {:?} since it has sources", path);
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    for path in recheck_addons {
        tokio::spawn(check_addon(path.clone(), workspace.clone()));
    }
}
