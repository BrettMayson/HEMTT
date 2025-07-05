use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::{WorkspacePath, addons::Addon, reporting::WorkspaceFiles};
use tokio::{sync::RwLock, task::JoinSet};
use tower_lsp::Client;
use tracing::{debug, warn};
use url::Url;

use crate::{
    diag_manager::DiagManager,
    files::FileCache,
    preprocessor::PreprocessorAnalyzer,
    workspace::{EditorWorkspace, EditorWorkspaces},
};

use super::SqfAnalyzer;

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

async fn check_addons(workspace: EditorWorkspace, database: Arc<Database>, client: Client) {
    debug!("sqf: checking addons");
    let mut futures = JoinSet::new();
    let mut next_addons = Vec::new();
    for addon in workspace.root().addons() {
        let Ok(source) = workspace.root().join(addon.as_str()) else {
            warn!("failed to join addon {:?}", addon);
            continue;
        };
        let location = if source.as_str().starts_with("/addons/") {
            hemtt_workspace::addons::Location::Addons
        } else {
            hemtt_workspace::addons::Location::Optionals
        };
        let name = source.as_str().split("/").nth(2).unwrap_or_default();
        let addon = Arc::new(
            Addon::new(workspace.root_disk(), name.to_string(), location)
                .expect("failed to create addon"),
        );
        for file in source.parent().walk_dir().unwrap_or_default() {
            if file.extension().unwrap_or_default() == "sqf"
                && !file.filename().contains(".inc.sqf")
            {
                futures.spawn(check_sqf(
                    file,
                    addon.clone(),
                    workspace.clone(),
                    database.clone(),
                ));
            }
        }
        next_addons.push(addon);
    }
    tokio::spawn(async move {
        futures.join_all().await;
        let Some(dm) = DiagManager::get() else {
            warn!("failed to get diag manager");
            return;
        };
        dm.sync("sqf");
        if let Err(e) = client.workspace_diagnostic_refresh().await {
            warn!("Failed to refresh diagnostics: {:?}", e);
        }
    });
}

async fn check_sqf(
    source: WorkspacePath,
    addon: Arc<Addon>,
    workspace: EditorWorkspace,
    database: Arc<Database>,
) {
    let Some(manager) = DiagManager::get() else {
        warn!("failed to get diag manager");
        return;
    };
    manager.clear_current(&format!("sqf:{}", source.as_str()));

    let mut lsp_diags = HashMap::new();
    PreprocessorAnalyzer::get()
        .mark_in_progress(source.clone())
        .await;
    let sources = match Processor::run(&source) {
        Ok(processed) => {
            {
                let workspace_files = WorkspaceFiles::new();
                match hemtt_sqf::parser::run(&database, &processed) {
                    Ok(sqf) => {
                        let (codes, report) = hemtt_sqf::analyze::analyze(
                            &sqf,
                            workspace.config().as_ref(),
                            &processed,
                            addon.clone(),
                            database,
                        );
                        if let Some(report) = report {
                            let cache = SqfAnalyzer::get();
                            let mut functions_defined = cache
                                .functions_defined
                                .entry(addon.name().to_string())
                                .or_insert_with(HashMap::new);
                            functions_defined.insert(
                                source.as_str().to_string(),
                                report.functions_defined().clone(),
                            );
                        }
                        for code in codes {
                            let Some(diag) = code.diagnostic() else {
                                warn!("failed to get diagnostic");
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
                    }
                    Err(hemtt_sqf::parser::ParserError::ParsingError(e)) => {
                        for error in e {
                            let Some(diag) = error.diagnostic() else {
                                continue;
                            };
                            let diag = diag.to_lsp(&workspace_files);
                            for (file, diag) in diag {
                                lsp_diags.entry(file).or_insert_with(Vec::new).push(diag);
                            }
                        }
                    }
                    Err(e) => {
                        for error in e.codes() {
                            let Some(diag) = error.diagnostic() else {
                                continue;
                            };
                            let diag = diag.to_lsp(&workspace_files);
                            for (file, diag) in diag {
                                lsp_diags.entry(file).or_insert_with(Vec::new).push(diag);
                            }
                        }
                    }
                }
            }
            let sources = processed.sources().into_iter().map(|(p, _)| p).collect();
            if FileCache::get().is_open(&workspace.to_url(&source)) {
                PreprocessorAnalyzer::get().save_processed(source.clone(), processed);
                PreprocessorAnalyzer::get().mark_done(source.clone()).await;
            }
            sources
        }
        Err((err_sources, err)) => {
            warn!("failed to parse sqf: {:?}", err);
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
            format!("sqf:{}", source.as_str()),
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

impl SqfAnalyzer {
    pub async fn check_lints(&self, workspace: EditorWorkspace, client: Client) {
        let database = self.get_database(&workspace).await;
        check_addons(workspace, database, client).await;
    }

    pub async fn partial_recheck_lints(&self, url: Url, client: Client) {
        let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(&url).await else {
            warn!("Failed to find workspace for {:?}", url);
            return;
        };
        let Ok(url_workspacepath) = workspace.join_url(&url) else {
            warn!(
                "Failed to join URL {:?} in workspace {:?}",
                url,
                workspace.url()
            );
            return;
        };
        let project_change = url.as_str().contains(".toml");
        let recheck_files = {
            let cache = Cache::get();
            let files = cache.files.read().await;
            files
                .iter()
                .filter_map(|(path, bundle)| {
                    if project_change {
                        return Some(path.clone());
                    }
                    if path == &url_workspacepath
                        || bundle.sources.iter().any(|source| {
                            workspace
                                .join_url(&url)
                                .map(|joined| joined == *source)
                                .unwrap_or(false)
                        })
                    {
                        Some(path.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };
        let database = self.get_database(&workspace).await;
        let mut futures = JoinSet::new();
        for path in recheck_files {
            let addon = Arc::new(
                Addon::new(
                    workspace.root_disk(),
                    path.as_str()
                        .split("/")
                        .nth(2)
                        .unwrap_or_default()
                        .to_string(),
                    if path.as_str().starts_with("/addons/") {
                        hemtt_workspace::addons::Location::Addons
                    } else {
                        hemtt_workspace::addons::Location::Optionals
                    },
                )
                .expect("failed to create addon"),
            );
            futures.spawn(check_sqf(
                path.clone(),
                addon,
                workspace.clone(),
                database.clone(),
            ));
        }
        tokio::spawn(async move {
            futures.join_all().await;
            let Some(dm) = DiagManager::get() else {
                warn!("failed to get diag manager");
                return;
            };
            dm.sync("sqf");
            if let Err(e) = client.workspace_diagnostic_refresh().await {
                warn!("Failed to refresh diagnostics: {:?}", e);
            }
        });
    }
}
