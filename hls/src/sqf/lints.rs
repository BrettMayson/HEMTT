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
    sources::SourceSync,
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

/// The [`Addon`] a workspace path belongs to, or `None` if it belongs to none.
fn addon_for(workspace: &EditorWorkspace, path: &WorkspacePath) -> Option<Arc<Addon>> {
    let location = if path.as_str().starts_with("/addons/") {
        hemtt_workspace::addons::Location::Addons
    } else if path.as_str().starts_with("/optionals/") {
        hemtt_workspace::addons::Location::Optionals
    } else {
        debug!("not linting `{path}`, not in `addons/` or `optionals/`");
        return None;
    };
    let Some(name) = path.as_str().split('/').nth(2).filter(|n| !n.is_empty()) else {
        debug!("not linting `{path}`, no addon name in path");
        return None;
    };
    match Addon::new(workspace.root_disk(), name.to_string(), location) {
        Ok(addon) => Some(Arc::new(addon)),
        Err(e) => {
            debug!("not linting `{path}`, failed to create addon: {e}");
            None
        }
    }
}

fn check_addons(workspace: &EditorWorkspace, database: &Arc<Database>, client: Client) {
    debug!("sqf: checking addons");
    let mut futures = JoinSet::new();
    for addon in workspace.root().addons() {
        let Ok(source) = workspace.root().join(addon.as_str()) else {
            warn!("failed to join addon {:?}", addon);
            continue;
        };
        let Some(addon) = addon_for(workspace, &source) else {
            continue;
        };
        for file in source.parent().walk_dir().unwrap_or_default() {
            if hemtt_sqf::is_compilation_unit(&file) {
                futures.spawn(check_sqf(
                    file,
                    addon.clone(),
                    workspace.clone(),
                    database.clone(),
                ));
            }
        }
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
    #[allow(clippy::or_fun_call)]
    let sources = match Processor::run_with_sources(
        &source,
        workspace
            .config()
            .as_ref()
            .map_or(&hemtt_common::config::PreprocessorOptions::default(), |f| {
                f.preprocessor()
            }),
        &SourceSync::get().database(),
    ) {
        Ok(processed) => {
            {
                let workspace_files = WorkspaceFiles::new();
                let checked = hemtt_sqf::check::check(
                    &processed,
                    workspace.config().as_ref(),
                    &addon,
                    database,
                );
                if let Some(report) = checked.report {
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
                for code in checked.codes {
                    let Some(diag) = code.diagnostic() else {
                        continue;
                    };
                    // a diagnostic inside a vendored include is not actionable
                    // from the project, so it is not shown
                    if diag.labels.iter().all(|l| l.file().is_include()) {
                        continue;
                    }
                    let lsp_diag = diag.to_lsp(&workspace_files);
                    for (file, diag) in lsp_diag {
                        lsp_diags.entry(file).or_insert_with(Vec::new).push(diag);
                    }
                }
            }
            let sources = processed.included_files().to_owned();
            if FileCache::get().is_open(&workspace.to_url(&source)) {
                PreprocessorAnalyzer::get().save_processed(source.clone(), processed);
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
                }
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
    if sources.is_empty() {
        cache.files.write().await.remove(&source);
    } else {
        cache
            .files
            .write()
            .await
            .insert(source.clone(), CacheBundle { sources });
    }
}

impl SqfAnalyzer {
    pub fn check_lints(&self, workspace: &EditorWorkspace, client: Client) {
        let database = match self.get_database(workspace) {
            Ok(database) => database,
            Err(e) => {
                warn!("not linting sqf, failed to build the command database: {e}");
                return;
            }
        };
        check_addons(workspace, &database, client);
    }

    pub async fn partial_recheck_lints(&self, url: Url, client: Client) {
        let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(&url).await else {
            warn!("Failed to find workspace for {:?}", url);
            return;
        };
        let Ok(saved) = workspace.join_url(&url) else {
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
            let mut recheck = files
                .iter()
                .filter_map(|(path, bundle)| {
                    if project_change {
                        return Some(path.clone());
                    }
                    if path == &saved
                        || bundle.sources.iter().any(|source| {
                            workspace
                                .join_url(&url)
                                .is_ok_and(|joined| joined == *source)
                        })
                    {
                        Some(path.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            drop(files);
            // `saved` is its own compilation unit, so it is always rechecked
            // directly, even when it isn't yet a known cache key - a new file,
            // the new path of a rename, or a first `didOpen`.
            if !recheck.contains(&saved) && hemtt_sqf::is_compilation_unit(&saved) {
                recheck.push(saved);
            }
            recheck
        };
        let database = match self.get_database(&workspace) {
            Ok(database) => database,
            Err(e) => {
                warn!("not linting sqf, failed to build the command database: {e}");
                return;
            }
        };
        let mut futures = JoinSet::new();
        for path in recheck_files {
            let Some(addon) = addon_for(&workspace, &path) else {
                continue;
            };
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

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::WorkspaceFolder;
    use url::Url;

    use super::addon_for;
    use crate::workspace::EditorWorkspace;

    /// Open a fixture folder under `hls/tests/fixtures/` as an editor workspace.
    fn fixture(name: &str) -> EditorWorkspace {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name);
        assert!(root.is_dir(), "missing fixture {}", root.display());
        EditorWorkspace::new(&WorkspaceFolder {
            uri: Url::from_directory_path(&root).expect("failed to build fixture url"),
            name: name.to_string(),
        })
        .expect("failed to open fixture workspace")
    }

    #[test]
    fn addon_in_addons() {
        let workspace = fixture("project");
        let path = workspace
            .root()
            .join("addons/valid/script.sqf")
            .expect("join");
        let addon = addon_for(&workspace, &path).expect("addons/valid is a valid addon");
        assert_eq!(addon.name(), "valid");
        assert_eq!(addon.location(), &hemtt_workspace::addons::Location::Addons);
    }

    #[test]
    fn addon_missing_prefix() {
        let workspace = fixture("project");
        let path = workspace
            .root()
            .join("addons/noprefix/script.sqf")
            .expect("join");
        assert!(addon_for(&workspace, &path).is_none());
    }

    #[test]
    fn no_addon_for_addons_root() {
        let workspace = fixture("project");
        let path = workspace.root().join("addons/stray.sqf").expect("join");
        assert!(addon_for(&workspace, &path).is_none());
    }

    /// Regression for #1307: a loose `.sqf` in an otherwise complete project.
    #[test]
    fn regression_1307_loose_sqf_in_valid_project() {
        let workspace = fixture("project");
        for file in ["tools/loose.sqf", "loose.sqf"] {
            let path = workspace.root().join(file).expect("join");
            assert!(
                addon_for(&workspace, &path).is_none(),
                "{file} should not resolve to an addon"
            );
        }
    }

    /// Regression for #1303: a folder that is not a HEMTT project at all.
    #[test]
    fn regression_1303_sqf_without_project() {
        let workspace = fixture("plain");
        let path = workspace.root().join("scripts/loose.sqf").expect("join");
        assert!(addon_for(&workspace, &path).is_none());
    }

    /// Regression for #1258: the old path of a rename, which no longer exists.
    #[test]
    fn regression_1258_renamed_sqf() {
        let workspace = fixture("plain");
        let path = workspace.root().join("scripts/gone.sqf").expect("join");
        assert!(!path.exists().expect("exists"));
        assert!(addon_for(&workspace, &path).is_none());
    }
}
