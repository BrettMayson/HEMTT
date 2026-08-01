//! Source database: a stable-identity, overlay-aware view of file content.
//!
//! Compiler internals (parsing, preprocessing, analysis) should read file
//! content through [`FileId`] + [`SourceDatabase`] rather than through
//! [`crate::WorkspacePath`] directly.
//!
//! This is what allows the same code to serve both:
//! - the CLI, where content always comes from the [`crate::Workspace`]/VFS, and
//! - an LSP, where an open/unsaved editor buffer should transparently take
//!   priority over the on-disk (or in-VFS) contents, without mutating the
//!   underlying workspace.
//!
//! [`FileId`] is a small, stable, `Copy` identifier for a logical file. The
//! same [`crate::WorkspacePath`] always maps to the same [`FileId`], so
//! query caches keyed by [`FileId`] remain valid across content edits (the
//! identity of the file does not change just because its content did).

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use crate::{Error, WorkspacePath, reporting::Token};

/// A stable identifier for a logical file, independent of whether its
/// current content comes from the workspace/VFS or from an LSP overlay
/// buffer.
///
/// `FileId`s are only comparable within the [`SourceDatabase`] that created
/// them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(u32);

/// Where the content of a [`SourceSnapshot`] was read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceOrigin {
    /// Read from the [`crate::Workspace`]/VFS (disk, memory layer, etc.)
    Workspace,
    /// Provided by an LSP overlay (an open/unsaved editor buffer)
    Overlay,
}

/// An immutable snapshot of a file's content at a point in time.
///
/// Cheap to clone: the content is reference counted.
#[derive(Debug, Clone)]
pub struct SourceSnapshot {
    content: Arc<str>,
    version: u64,
    origin: SourceOrigin,
}

impl SourceSnapshot {
    #[must_use]
    /// The file's content
    pub const fn content(&self) -> &Arc<str> {
        &self.content
    }

    #[must_use]
    /// A monotonic version number.
    ///
    /// For [`SourceOrigin::Overlay`] snapshots this is caller-supplied (e.g.
    /// the LSP document version). For [`SourceOrigin::Workspace`] snapshots
    /// this is always `0`, since the workspace/VFS does not track versions.
    pub const fn version(&self) -> u64 {
        self.version
    }

    #[must_use]
    /// Where this snapshot's content came from
    pub const fn origin(&self) -> SourceOrigin {
        self.origin
    }
}

#[derive(Debug, Clone)]
struct Overlay {
    content: Arc<str>,
    version: u64,
}

/// Memoized `parse(file)` query result: the [`SourceSnapshot`] version it
/// was computed from, and the resulting tokens.
type CachedTokens = (u64, Arc<Vec<Arc<Token>>>);

#[derive(Debug, Default)]
struct Inner {
    ids: HashMap<WorkspacePath, FileId>,
    paths: HashMap<FileId, WorkspacePath>,
    overlays: HashMap<FileId, Overlay>,
    /// Memoized `parse(file)` query results, keyed by the [`FileId`] and the
    /// [`SourceSnapshot`] version they were computed from. A version
    /// mismatch (e.g. an LSP edit bumping the overlay version) invalidates
    /// the entry naturally on the next lookup.
    tokens: HashMap<FileId, CachedTokens>,
    /// Forward include edges: file -> the files it directly `#include`s.
    dependencies: HashMap<FileId, HashSet<FileId>>,
    /// Reverse include edges: file -> the files that directly `#include` it.
    dependents: HashMap<FileId, HashSet<FileId>>,
    next: u32,
}

#[derive(Debug, Clone, Default)]
/// A [`FileId`]-addressed, overlay-aware view over file content.
///
/// This is the boundary between the [`crate::Workspace`]/VFS and compiler
/// internals. Compiler code should be written against [`FileId`] +
/// [`SourceDatabase`], and should not need to know whether a given file's
/// content came from disk/VFS or from an LSP editor buffer.
///
/// A [`SourceDatabase`] is cheap to clone (it is reference counted) and is
/// safe to share across threads.
pub struct SourceDatabase {
    inner: Arc<RwLock<Inner>>,
}

impl SourceDatabase {
    #[must_use]
    /// Create a new, empty source database
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    /// Get or create the stable [`FileId`] for a [`WorkspacePath`].
    ///
    /// The same workspace path always returns the same [`FileId`] for the
    /// lifetime of this database.
    pub fn file_id(&self, path: &WorkspacePath) -> FileId {
        let mut inner = self.inner.write().expect("SourceDatabase lock poisoned");
        if let Some(id) = inner.ids.get(path) {
            return *id;
        }
        let id = FileId(inner.next);
        inner.next += 1;
        inner.ids.insert(path.clone(), id);
        inner.paths.insert(id, path.clone());
        id
    }

    #[must_use]
    /// Get the [`WorkspacePath`] a [`FileId`] was created from, if it is
    /// still known to this database.
    pub fn path(&self, id: FileId) -> Option<WorkspacePath> {
        self.inner
            .read()
            .expect("SourceDatabase lock poisoned")
            .paths
            .get(&id)
            .cloned()
    }

    /// Set (or replace) the overlay content for a file, e.g. in response to
    /// an LSP `didOpen`/`didChange` notification.
    ///
    /// This never touches the underlying [`crate::Workspace`]/VFS.
    ///
    /// Normalizes CRLF line endings to LF, matching
    /// [`WorkspacePath::read_to_string`] so that content behaves the same
    /// regardless of origin.
    pub fn set_overlay(&self, id: FileId, content: impl AsRef<str>, version: u64) {
        let normalized: Arc<str> = Arc::from(content.as_ref().replace('\r', ""));
        let mut inner = self.inner.write().expect("SourceDatabase lock poisoned");
        inner.overlays.insert(
            id,
            Overlay {
                content: normalized,
                version,
            },
        );
    }

    /// Remove the overlay for a file, e.g. in response to an LSP
    /// `didClose` notification.
    ///
    /// Subsequent reads fall back to the workspace/VFS contents.
    pub fn clear_overlay(&self, id: FileId) {
        self.inner
            .write()
            .expect("SourceDatabase lock poisoned")
            .overlays
            .remove(&id);
    }

    #[must_use]
    /// Whether the given file currently has overlay content
    pub fn has_overlay(&self, id: FileId) -> bool {
        self.inner
            .read()
            .expect("SourceDatabase lock poisoned")
            .overlays
            .contains_key(&id)
    }

    /// Read the current [`SourceSnapshot`] for a file.
    ///
    /// Prefers overlay content if present, otherwise falls back to the
    /// workspace/VFS.
    ///
    /// # Errors
    /// - [`Error::UnknownFileId`] if the id was not created by this database
    /// - Any error from reading the workspace/VFS if there is no overlay
    pub fn source(&self, id: FileId) -> Result<SourceSnapshot, Error> {
        {
            let inner = self.inner.read().expect("SourceDatabase lock poisoned");
            if let Some(overlay) = inner.overlays.get(&id) {
                return Ok(SourceSnapshot {
                    content: overlay.content.clone(),
                    version: overlay.version,
                    origin: SourceOrigin::Overlay,
                });
            }
        }
        let path = self.path(id).ok_or(Error::UnknownFileId)?;
        let content = path.read_to_string()?;
        Ok(SourceSnapshot {
            content: Arc::from(content),
            version: 0,
            origin: SourceOrigin::Workspace,
        })
    }

    /// Convenience: get or create the [`FileId`] for a path and read its
    /// current snapshot in one call.
    ///
    /// # Errors
    /// See [`SourceDatabase::source`]
    pub fn source_for_path(&self, path: &WorkspacePath) -> Result<(FileId, SourceSnapshot), Error> {
        let id = self.file_id(path);
        let snapshot = self.source(id)?;
        Ok((id, snapshot))
    }

    /// Memoized `parse(file)` query.
    ///
    /// Returns the cached token stream for `id` if one was computed at the
    /// same `version`, otherwise runs `compute` and caches the result.
    ///
    /// This makes the parse step reusable across:
    /// - repeated includes of the same file under the same content version
    ///   (e.g. `common.hpp` included by both `A.cpp` and `B.cpp`), and
    /// - LSP re-preprocessing that did not change this particular file.
    ///
    /// # Errors
    /// Whatever `compute` returns
    pub fn get_or_parse<E>(
        &self,
        id: FileId,
        version: u64,
        compute: impl FnOnce() -> Result<Vec<Arc<Token>>, E>,
    ) -> Result<Arc<Vec<Arc<Token>>>, E> {
        {
            let inner = self.inner.read().expect("SourceDatabase lock poisoned");
            if let Some((cached_version, tokens)) = inner.tokens.get(&id)
                && *cached_version == version
            {
                return Ok(tokens.clone());
            }
        }
        let tokens = Arc::new(compute()?);
        self.inner
            .write()
            .expect("SourceDatabase lock poisoned")
            .tokens
            .insert(id, (version, tokens.clone()));
        Ok(tokens)
    }

    /// Record that `from` directly `#include`s `to`.
    ///
    /// Maintains both the forward (`from` -> `to`) and reverse (`to` ->
    /// `from`) edges, so that incremental re-preprocessing can answer both
    /// "what does this file include" and "what includes this file".
    pub fn record_dependency(&self, from: FileId, to: FileId) {
        let mut inner = self.inner.write().expect("SourceDatabase lock poisoned");
        inner.dependencies.entry(from).or_default().insert(to);
        inner.dependents.entry(to).or_default().insert(from);
    }

    #[must_use]
    /// The set of files directly `#include`d by `id`, in the most recent
    /// preprocessing run that recorded dependencies for it.
    pub fn dependencies_of(&self, id: FileId) -> Vec<FileId> {
        self.inner
            .read()
            .expect("SourceDatabase lock poisoned")
            .dependencies
            .get(&id)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    #[must_use]
    /// The set of files that directly `#include` `id`, in the most recent
    /// preprocessing run(s) that recorded dependencies involving it.
    pub fn dependents_of(&self, id: FileId) -> Vec<FileId> {
        self.inner
            .read()
            .expect("SourceDatabase lock poisoned")
            .dependents
            .get(&id)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Clear the recorded forward dependency edges for `id` (e.g. before
    /// re-preprocessing it from scratch), removing the corresponding
    /// reverse edges too.
    ///
    /// This does not clear `id`'s reverse edges (who depends on `id`),
    /// since those are owned by the files that include `id`, not by `id`
    /// itself.
    pub fn clear_dependencies_of(&self, id: FileId) {
        let mut inner = self.inner.write().expect("SourceDatabase lock poisoned");
        if let Some(old) = inner.dependencies.remove(&id) {
            for dependency in old {
                if let Some(back) = inner.dependents.get_mut(&dependency) {
                    back.remove(&id);
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::io::Write;

    fn memory_workspace() -> WorkspacePath {
        crate::Workspace::builder()
            .memory()
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .unwrap()
    }

    #[test]
    fn file_id_is_stable_for_same_path() {
        let workspace = memory_workspace();
        let path = workspace.join("test.hpp").unwrap();
        let db = SourceDatabase::new();
        let first = db.file_id(&path);
        let second = db.file_id(&path);
        assert_eq!(first, second);
    }

    #[test]
    fn file_id_differs_for_different_paths() {
        let workspace = memory_workspace();
        let a = workspace.join("a.hpp").unwrap();
        let b = workspace.join("b.hpp").unwrap();
        let db = SourceDatabase::new();
        assert_ne!(db.file_id(&a), db.file_id(&b));
    }

    #[test]
    fn falls_back_to_workspace_when_no_overlay() {
        let workspace = memory_workspace();
        let path = workspace.join("test.hpp").unwrap();
        path.create_file()
            .unwrap()
            .write_all(b"value = 1;")
            .unwrap();
        let db = SourceDatabase::new();
        let (id, snapshot) = db.source_for_path(&path).unwrap();
        assert_eq!(snapshot.origin(), SourceOrigin::Workspace);
        assert_eq!(&**snapshot.content(), "value = 1;");
        assert!(!db.has_overlay(id));
    }

    #[test]
    fn overlay_takes_priority_over_workspace() {
        let workspace = memory_workspace();
        let path = workspace.join("test.hpp").unwrap();
        path.create_file()
            .unwrap()
            .write_all(b"value = 1;")
            .unwrap();
        let db = SourceDatabase::new();
        let id = db.file_id(&path);
        db.set_overlay(id, "value = 2;", 1);
        let snapshot = db.source(id).unwrap();
        assert_eq!(snapshot.origin(), SourceOrigin::Overlay);
        assert_eq!(&**snapshot.content(), "value = 2;");
        assert_eq!(snapshot.version(), 1);
    }

    #[test]
    fn closing_overlay_falls_back_to_workspace() {
        let workspace = memory_workspace();
        let path = workspace.join("test.hpp").unwrap();
        path.create_file()
            .unwrap()
            .write_all(b"disk contents")
            .unwrap();
        let db = SourceDatabase::new();
        let id = db.file_id(&path);
        db.set_overlay(id, "unsaved contents", 1);
        assert_eq!(&**db.source(id).unwrap().content(), "unsaved contents");

        db.clear_overlay(id);
        assert!(!db.has_overlay(id));
        assert_eq!(&**db.source(id).unwrap().content(), "disk contents");
    }

    #[test]
    fn unknown_file_id_is_an_error() {
        let db = SourceDatabase::new();
        let other = SourceDatabase::new();
        let workspace = memory_workspace();
        let path = workspace.join("test.hpp").unwrap();
        let id = other.file_id(&path);
        assert!(matches!(db.source(id), Err(Error::UnknownFileId)));
    }

    fn dummy_token(path: &WorkspacePath) -> Token {
        use crate::position::{LineCol, Position};
        use crate::reporting::Symbol;
        Token::new(
            Symbol::Word("x".to_string()),
            Position::new(LineCol(0, (1, 0)), LineCol(1, (1, 1)), path.clone()),
        )
    }

    #[test]
    fn get_or_parse_computes_once_per_version() {
        let workspace = memory_workspace();
        let path = workspace.join("test.hpp").unwrap();
        let db = SourceDatabase::new();
        let id = db.file_id(&path);
        let calls = std::sync::atomic::AtomicUsize::new(0);

        let compute = |db: &SourceDatabase| {
            db.get_or_parse::<()>(id, 1, || {
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(vec![Arc::new(dummy_token(&path))])
            })
            .unwrap()
        };

        let first = compute(&db);
        let second = compute(&db);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn get_or_parse_recomputes_on_version_change() {
        let workspace = memory_workspace();
        let path = workspace.join("test.hpp").unwrap();
        let db = SourceDatabase::new();
        let id = db.file_id(&path);
        let calls = std::sync::atomic::AtomicUsize::new(0);

        let compute = |db: &SourceDatabase, version: u64| {
            db.get_or_parse::<()>(id, version, || {
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(vec![Arc::new(dummy_token(&path))])
            })
            .unwrap()
        };

        compute(&db, 1);
        compute(&db, 2);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn record_dependency_tracks_forward_and_reverse_edges() {
        let workspace = memory_workspace();
        let root = workspace.join("root.hpp").unwrap();
        let included = workspace.join("included.hpp").unwrap();
        let db = SourceDatabase::new();
        let root_id = db.file_id(&root);
        let included_id = db.file_id(&included);

        db.record_dependency(root_id, included_id);

        assert_eq!(db.dependencies_of(root_id), vec![included_id]);
        assert_eq!(db.dependents_of(included_id), vec![root_id]);
        assert!(db.dependencies_of(included_id).is_empty());
        assert!(db.dependents_of(root_id).is_empty());
    }

    #[test]
    fn record_dependency_supports_multiple_edges() {
        let workspace = memory_workspace();
        let root = workspace.join("root.hpp").unwrap();
        let a = workspace.join("a.hpp").unwrap();
        let b = workspace.join("b.hpp").unwrap();
        let db = SourceDatabase::new();
        let root_id = db.file_id(&root);
        let a_id = db.file_id(&a);
        let b_id = db.file_id(&b);

        db.record_dependency(root_id, a_id);
        db.record_dependency(root_id, b_id);
        // b.hpp is also included by a.hpp
        db.record_dependency(a_id, b_id);

        let mut deps = db.dependencies_of(root_id);
        deps.sort();
        let mut expected = vec![a_id, b_id];
        expected.sort();
        assert_eq!(deps, expected);

        let mut dependents = db.dependents_of(b_id);
        dependents.sort();
        let mut expected = vec![root_id, a_id];
        expected.sort();
        assert_eq!(dependents, expected);
    }

    #[test]
    fn clear_dependencies_of_removes_forward_and_matching_reverse_edges() {
        let workspace = memory_workspace();
        let root = workspace.join("root.hpp").unwrap();
        let included = workspace.join("included.hpp").unwrap();
        let db = SourceDatabase::new();
        let root_id = db.file_id(&root);
        let included_id = db.file_id(&included);

        db.record_dependency(root_id, included_id);
        db.clear_dependencies_of(root_id);

        assert!(db.dependencies_of(root_id).is_empty());
        assert!(db.dependents_of(included_id).is_empty());
    }
}
