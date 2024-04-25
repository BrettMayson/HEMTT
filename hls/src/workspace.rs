use std::{
    collections::HashMap,
    mem::MaybeUninit,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use hemtt_common::project::hemtt::PDriveOption;
use hemtt_workspace::{LayerType, Workspace, WorkspacePath};
use tower_lsp::lsp_types::{DidChangeWorkspaceFoldersParams, WorkspaceFolder};
use tracing::debug;
use url::Url;

#[derive(Clone)]
pub struct EditorWorkspaces {
    workspaces: Arc<RwLock<HashMap<Url, EditorWorkspace>>>,
}

impl EditorWorkspaces {
    pub fn get() -> Self {
        static mut SINGLETON: MaybeUninit<EditorWorkspaces> = MaybeUninit::uninit();
        static mut INIT: bool = false;
        unsafe {
            if !INIT {
                SINGLETON = MaybeUninit::new(Self {
                    workspaces: Arc::new(RwLock::new(HashMap::new())),
                });
                INIT = true;
            }
            SINGLETON.assume_init_ref().clone()
        }
    }

    pub fn initialize(&self, folders: Vec<WorkspaceFolder>) {
        let mut workspaces = self.workspaces.write().unwrap();
        for folder in folders {
            debug!("adding workspace {}", folder.uri);
            if let Some(workspace) = EditorWorkspace::new(&folder) {
                workspaces.insert(folder.uri.clone(), workspace);
            } else {
                debug!("failed to add workspace {}", folder.uri);
            }
        }
    }

    pub fn changed(&self, changes: DidChangeWorkspaceFoldersParams) {
        let mut workspaces = self.workspaces.write().unwrap();
        for removed in changes.event.removed {
            if workspaces.contains_key(&removed.uri) {
                workspaces.remove(&removed.uri);
            }
        }
        for added in changes.event.added {
            if workspaces.contains_key(&added.uri) {
                workspaces.remove(&added.uri);
            }
            debug!("adding workspace {}", added.uri);
            if let Some(workspace) = EditorWorkspace::new(&added) {
                workspaces.insert(added.uri.clone(), workspace);
            } else {
                debug!("failed to add workspace {}", added.uri);
            }
        }
    }

    pub fn guess_workspace(&self, uri: &Url) -> Option<EditorWorkspace> {
        let workspaces = self.workspaces.read().unwrap();
        let mut best = None;
        let mut best_len = 0;
        for (folder, workspace) in workspaces.iter() {
            let path = folder.path();
            let uri_path = uri.path();
            let len = path
                .chars()
                .zip(uri_path.chars())
                .take_while(|(a, b)| a == b)
                .count();
            if len > best_len {
                best = Some(workspace.clone());
                best_len = len;
            }
        }
        best
    }
}

#[derive(Clone)]
pub struct EditorWorkspace {
    path: Url,
    workspace: WorkspacePath,
}

impl EditorWorkspace {
    pub fn new(folder: &WorkspaceFolder) -> Option<Self> {
        if folder.uri.scheme() == "file" {
            let Ok(workspace) = Workspace::builder()
                .physical(
                    &PathBuf::from(format!("{}", folder.uri).replace("file://", "")),
                    LayerType::Source,
                )
                .finish(None, true, &PDriveOption::Disallow)
            else {
                return None;
            };
            Some(Self {
                workspace,
                path: folder.uri.clone(),
            })
        } else {
            None
        }
    }

    pub fn join_url(&self, url: &Url) -> Result<WorkspacePath, String> {
        let path = url.path().strip_prefix(self.path.path()).unwrap();
        self.workspace.join(path).map_err(|e| format!("{}", e))
    }
}
