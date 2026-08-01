mod goto;
mod signature;

use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use hemtt_preprocessor::Processor;
use hemtt_workspace::{
    WorkspacePath,
    reporting::{CacheProcessed, Processed},
};
use tracing::warn;
use url::Url;

use crate::{
    sources::SourceSync,
    workspace::{EditorWorkspace, EditorWorkspaces},
};

#[derive(Clone)]
pub struct PreprocessorAnalyzer {
    /// The most recent preprocessed output for each addon (keyed by the
    /// addon's root `config.cpp`) or `.sqf` file. Populated by the
    /// whole-addon lint scans. Used for hover-style features (goto
    /// definition, signature help) where briefly stale data while a scan is
    /// in flight is acceptable.
    processed: Arc<DashMap<WorkspacePath, CacheProcessed>>,
}

impl PreprocessorAnalyzer {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<PreprocessorAnalyzer> = LazyLock::new(|| PreprocessorAnalyzer {
            processed: Arc::new(DashMap::new()),
        });
        (*SINGLETON).clone()
    }

    pub fn save_processed(&self, source: WorkspacePath, processed: Processed) {
        self.processed.insert(source, processed.cache());
    }

    pub async fn on_close(&self, url: &Url) {
        let Some(workspace) = EditorWorkspaces::get().guess_workspace_retry(url).await else {
            warn!("Failed to find workspace for {:?}", url);
            return;
        };
        let Ok(source) = workspace.join_url(url) else {
            warn!("Failed to join url {:?}", url);
            return;
        };
        if source.extension().as_deref() == Some("sqf") && self.processed.remove(&source).is_some()
        {
            tracing::debug!("sqf: removed processed cache for {}", source);
        }
    }

    /// Preprocess `url` on demand and return its clean, macro-expanded
    /// output, for the `hemtt/processed` preview command.
    ///
    /// This always recomputes from the current (possibly unsaved) buffer
    /// content through [`SourceSync`], rather than relying on a background
    /// lint scan having already populated a cache, so the preview is never
    /// stale or racy with an in-flight save.
    pub async fn get_processed(&self, url: Url) -> Option<String> {
        let workspace = EditorWorkspaces::get().guess_workspace_retry(&url).await?;
        let source = workspace.join_url(&url).ok()?;
        let root = resolve_processing_root(&workspace, &source);
        let config = workspace.config();
        #[allow(clippy::or_fun_call)]
        match Processor::run_with_sources(
            &root,
            config
                .as_ref()
                .map_or(&hemtt_common::config::PreprocessorOptions::default(), |f| {
                    f.preprocessor()
                }),
            &SourceSync::get().database(),
        ) {
            Ok(processed) => Some(processed.cache().output),
            Err((_, e)) => {
                warn!("failed to preprocess {:?}: {:?}", root, e);
                None
            }
        }
    }
}

/// Config headers (`.hpp`/`.ext`) aren't independently preprocessable, they
/// only make sense in the context of the addon's root `config.cpp` that
/// includes them, while `.sqf` files are always self-contained. Resolves
/// `source` to whichever file should actually be run through the
/// preprocessor to preview it.
fn resolve_processing_root(workspace: &EditorWorkspace, source: &WorkspacePath) -> WorkspacePath {
    if source.extension().as_deref() == Some("sqf") {
        return source.clone();
    }
    workspace
        .root()
        .addons()
        .iter()
        .filter_map(|config| workspace.root().join(config.as_str()).ok())
        .find(|config| {
            config.as_str() == source.as_str()
                || source.as_str().starts_with(config.parent().as_str())
        })
        .unwrap_or_else(|| source.clone())
}
