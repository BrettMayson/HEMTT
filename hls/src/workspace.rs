use std::{
    collections::HashMap,
    mem::MaybeUninit,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc, RwLock},
};

use hemtt_common::project::hemtt::PDriveOption;
use hemtt_workspace::{LayerType, Workspace, WorkspacePath};
use tower_lsp::{
    lsp_types::{DidChangeWorkspaceFoldersParams, WorkspaceFolder},
    Client,
};
use tracing::debug;
use url::Url;

#[derive(Clone)]
pub struct EditorWorkspaces {
    workspaces: Arc<RwLock<HashMap<Url, EditorWorkspace>>>,
}

impl EditorWorkspaces {
    pub fn get() -> Self {
        static mut SINGLETON: MaybeUninit<EditorWorkspaces> = MaybeUninit::uninit();
        static mut INIT: AtomicBool = AtomicBool::new(false);
        unsafe {
            if !INIT.swap(true, std::sync::atomic::Ordering::SeqCst) {
                SINGLETON = MaybeUninit::new(Self {
                    workspaces: Arc::new(RwLock::new(HashMap::new())),
                });
                INIT.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            SINGLETON.assume_init_ref().clone()
        }
    }

    pub fn initialize(&self, folders: Vec<WorkspaceFolder>) {
        let mut workspaces = self.workspaces.write().unwrap();
        for folder in folders {
            debug!("adding workspace {}", folder.uri);
            if let Some(workspace) = EditorWorkspace::new(&folder) {
                workspaces.insert(folder.uri.clone(), workspace.clone());
                tokio::spawn(crate::config::workspace_added(workspace));
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
                workspaces.insert(added.uri.clone(), workspace.clone());
                tokio::spawn(crate::config::workspace_added(workspace));
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

    pub async fn guess_workspace_retry(&self, uri: &Url) -> Option<EditorWorkspace> {
        let mut tries = 5;
        loop {
            if let Some(workspace) = self.guess_workspace(uri) {
                break Some(workspace);
            } else {
                tries -= 1;
                if tries == 0 {
                    return None;
                }
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        }
    }
}

#[derive(Clone)]
pub struct EditorWorkspace {
    url: Url,
    workspace: WorkspacePath,
}

impl EditorWorkspace {
    pub fn new(folder: &WorkspaceFolder) -> Option<Self> {
        if folder.uri.scheme() == "file" {
            let Ok(workspace) = Workspace::builder()
                .physical(
                    &PathBuf::from(
                        urlencoding::decode(
                            folder
                                .uri
                                .to_string()
                                .replace(if cfg!(windows) { "file:///" } else { "file://" }, "")
                                .as_str(),
                        )
                        .expect("Failed to decode URL")
                        .to_string(),
                    ),
                    LayerType::Source,
                )
                .finish(None, true, &PDriveOption::Disallow)
            else {
                return None;
            };
            Some(Self {
                workspace,
                url: folder.uri.clone(),
            })
        } else {
            None
        }
    }

    pub fn join_url(&self, url: &Url) -> Result<WorkspacePath, String> {
        let path = url.path().strip_prefix(self.url.path()).unwrap();
        self.workspace.join(path).map_err(|e| format!("{}", e))
    }

    pub fn to_url(&self, path: &WorkspacePath) -> Url {
        // trim the workspace path
        let path = path.as_str().strip_prefix(self.workspace.as_str()).unwrap();
        let path = path.replace('\\', "/");
        // url encode the path
        let path = urlencoding::encode(&path);
        let path = path.replace("%2F", "/");
        let mut url = self.url.clone();
        url.set_path(format!("{}{}", url.path(), path).as_str());
        url
    }

    pub fn root(&self) -> &WorkspacePath {
        &self.workspace
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}
