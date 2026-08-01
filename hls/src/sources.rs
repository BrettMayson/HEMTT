//! Keeps a shared [`SourceDatabase`] in sync with open editor buffers.
//!
//! This is the single place where LSP `didOpen`/`didChange`/`didClose`
//! events become overlay updates. Every part of the language server that
//! needs to preprocess or parse a file (config lints, sqf lints, sqf
//! compile preview, ...) should go through [`SourceSync::database`] /
//! [`hemtt_preprocessor::Processor::run_with_sources`] rather than
//! [`hemtt_preprocessor::Processor::run`], so that unsaved buffer content is
//! always what gets analyzed, without ever touching the on-disk workspace.

use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use hemtt_workspace::{FileId, SourceDatabase, WorkspacePath};
use tracing::warn;
use url::Url;

use crate::workspace::EditorWorkspaces;

#[derive(Clone)]
pub struct SourceSync {
    sources: SourceDatabase,
    /// Cache of previously resolved `Url -> (WorkspacePath, FileId)`, so
    /// repeated edits to the same open buffer don't need to re-guess its
    /// workspace on every keystroke.
    resolved: Arc<DashMap<Url, (WorkspacePath, FileId)>>,
}

impl SourceSync {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<SourceSync> = LazyLock::new(|| SourceSync {
            sources: SourceDatabase::new(),
            resolved: Arc::new(DashMap::new()),
        });
        (*SINGLETON).clone()
    }

    #[must_use]
    /// The shared, overlay-aware source database. Pass this to
    /// [`hemtt_preprocessor::Processor::run_with_sources`] wherever a
    /// `WorkspacePath` would otherwise be preprocessed from disk.
    pub fn database(&self) -> SourceDatabase {
        self.sources.clone()
    }

    /// Resolve a `Url` to its `(WorkspacePath, FileId)` in the shared
    /// [`SourceDatabase`], retrying briefly if the owning workspace hasn't
    /// been registered yet.
    pub async fn resolve(&self, url: &Url) -> Option<(WorkspacePath, FileId)> {
        if let Some(entry) = self.resolved.get(url) {
            return Some(entry.clone());
        }
        let workspace = EditorWorkspaces::get().guess_workspace_retry(url).await?;
        let path = workspace.join_url(url).ok()?;
        let id = self.sources.file_id(&path);
        self.resolved.insert(url.clone(), (path.clone(), id));
        Some((path, id))
    }

    /// Update the overlay for `url` with the current full buffer content,
    /// keeping the underlying workspace/VFS untouched.
    ///
    /// `version` should be a monotonically increasing value for the
    /// document (e.g. the LSP document version); a version that hasn't
    /// changed since the last call is treated as a no-op by memoized
    /// queries downstream (see [`SourceDatabase::get_or_parse`]).
    pub async fn on_change(&self, url: &Url, content: &str, version: i32) {
        let Some((_, id)) = self.resolve(url).await else {
            warn!("SourceSync: failed to resolve workspace for {url}");
            return;
        };
        self.sources
            .set_overlay(id, content, u64::try_from(version.max(0)).unwrap_or(0));
    }

    /// Remove the overlay for `url` (the buffer was closed), falling back
    /// to the workspace/VFS content on subsequent reads.
    pub fn on_close(&self, url: &Url) {
        if let Some((_, (_, id))) = self.resolved.remove(url) {
            self.sources.clear_overlay(id);
        }
    }
}
