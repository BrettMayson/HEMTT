use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use hemtt_preprocessor::Processor;
use hemtt_workspace::{WorkspacePath, reporting::WorkspaceFiles};
use tokio::{sync::RwLock, task::JoinSet};
use tower_lsp::Client;
use tracing::{debug, warn};
use url::Url;

use crate::{
    config::ConfigAnalyzer,
    diag_manager::DiagManager,
    preprocessor::PreprocessorAnalyzer,
    workspace::{EditorWorkspace, EditorWorkspaces},
};

struct CacheBundle {
    pub sources: Vec<WorkspacePath>,
}

#[derive(Clone)]
struct Cache {
    files: Arc<RwLock<HashMap<WorkspacePath, CacheBundle>>>,
}

impl Cache {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<Cache> = LazyLock::new(|| Cache {
            files: Arc::new(RwLock::new(HashMap::new())),
        });
        (*SINGLETON).clone()
    }
}

async fn check_addons(workspace: EditorWorkspace, client: Client) {
    let mut futures = JoinSet::new();
    for config in workspace.root().addons() {
        let Ok(source) = workspace.root().join(config.as_str()) else {
            warn!("failed to join config {:?}", config);
            continue;
        };
        futures.spawn(check_addon(source, workspace.clone()));
    }
    tokio::spawn(async move {
        futures.join_all().await;
        let Some(dm) = DiagManager::get() else {
            warn!("failed to get diag manager");
            return;
        };
        dm.sync("config");
        if let Err(e) = client.workspace_diagnostic_refresh().await {
            warn!("Failed to refresh diagnostics: {:?}", e);
        }
    });
}

async fn check_addon(source: WorkspacePath, workspace: EditorWorkspace) {
    let Some(manager) = DiagManager::get() else {
        warn!("failed to get diag manager");
        return;
    };
    manager.clear_current(&format!("config:{}", source.as_str()));
    let mut lsp_diags = HashMap::new();
    PreprocessorAnalyzer::get()
        .mark_in_progress(source.clone())
        .await;
    let sources = match Processor::run(
        &source,
        workspace
            .config()
            .as_ref()
            .map_or(&hemtt_common::config::PreprocessorOptions::default(), |f| {
                f.preprocessor()
            }),
    ) {
        Ok(processed) => {
            {
                let workspace_files = WorkspaceFiles::new();
                match hemtt_config::parse(workspace.config().as_ref(), &processed) {
                    Ok(report) => {
                        for code in report.warnings().iter().chain(report.errors().iter()) {
                            warn!("code: {:?}", code);
                            let Some(diag) = code.diagnostic() else {
                                continue;
                            };
                            if diag.labels.iter().all(|l| l.file().is_include()) {
                                continue;
                            }
                            let lsp_diag = diag.to_lsp(&workspace_files);
                            for (file, diag) in lsp_diag {
                                lsp_diags.entry(file).or_insert_with(Vec::new).push(diag);
                            }
                        }
                        let config_analyzer = ConfigAnalyzer::get();
                        config_analyzer.functions_defined.insert(
                            {
                                // `/folder/addon/blah` => addon
                                let parts: Vec<&str> = source.as_str().split('/').collect();
                                if parts.len() < 3 {
                                    warn!("Invalid config path: {}", source.as_str());
                                    if parts.len() == 2 {
                                        parts[1].to_string()
                                    } else {
                                        source.as_str().to_string()
                                    }
                                } else {
                                    parts[2].to_string()
                                }
                            },
                            report.functions_defined().clone(),
                        );
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
            }
            let sources = processed.sources().into_iter().map(|(p, _)| p).collect();
            PreprocessorAnalyzer::get().save_processed(source.parent(), processed);
            PreprocessorAnalyzer::get().mark_done(source.clone()).await;
            sources
        }
        Err((err_sources, err)) => {
            warn!("failed to parse config: {:?}", err);
            debug!("failed sources: {:?}", err_sources);
            if let hemtt_preprocessor::Error::Code(code) = err {
                let workspace_files = WorkspaceFiles::new();
                if let Some(diag) = code.diagnostic() {
                    let lsp_diag = diag.to_lsp(&workspace_files);
                    for (file, diag) in lsp_diag {
                        lsp_diags.entry(file).or_insert_with(Vec::new).push(diag);
                    }
                };
            }
            err_sources
        }
    };
    for (file, diags) in lsp_diags {
        manager.set_current(
            format!("config:{}", source.as_str()),
            &workspace.to_url(&file),
            diags,
        );
    }
    let cache = Cache::get();
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

pub async fn workspace_added(workspace: EditorWorkspace, client: Client) {
    tokio::spawn(check_addons(workspace, client));
}

pub async fn process(url: Url, client: Client) {
    let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(&url).await else {
        warn!("Failed to find workspace for {:?}", url);
        return;
    };
    let Ok(saved) = workspace.join_url(&url) else {
        warn!("failed to join url {:?}", url);
        return;
    };
    let project_change = url.as_str().contains(".toml");
    let recheck_addons = {
        let cache = Cache::get();
        let files = cache.files.read().await;
        files
            .iter()
            .filter_map(|(path, bundle)| {
                if project_change {
                    return Some(path.clone());
                }
                if &saved == path || bundle.sources.contains(&saved) {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    let mut futures = JoinSet::new();
    for path in recheck_addons {
        futures.spawn(check_addon(path.clone(), workspace.clone()));
    }
    tokio::spawn(async move {
        futures.join_all().await;
        let Some(dm) = DiagManager::get() else {
            warn!("failed to get diag manager");
            return;
        };
        dm.sync("config");
        if let Err(e) = client.workspace_diagnostic_refresh().await {
            warn!("Failed to refresh diagnostics: {:?}", e);
        }
    });
}
