use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
};

use hemtt_common::config::{PDriveOption, ProjectConfig};
use hemtt_workspace::{LayerType, Workspace, WorkspacePath};
use tower_lsp::{
    Client,
    lsp_types::{DidChangeWorkspaceFoldersParams, WorkspaceFolder},
};
use tracing::debug;
use url::Url;

use crate::{config::ConfigAnalyzer, sqf::SqfAnalyzer};

#[derive(Clone)]
pub struct EditorWorkspaces {
    workspaces: Arc<RwLock<HashMap<Url, EditorWorkspace>>>,
}

impl EditorWorkspaces {
    pub fn get() -> Self {
        static SINGLETON: LazyLock<EditorWorkspaces> = LazyLock::new(|| EditorWorkspaces {
            workspaces: Arc::new(RwLock::new(HashMap::new())),
        });
        (*SINGLETON).clone()
    }

    pub fn initialize(&self, folders: Vec<WorkspaceFolder>, client: &Client) {
        let mut workspaces = self
            .workspaces
            .write()
            .expect("Failed to lock workspaces for writing");
        for folder in folders {
            debug!("adding workspace {}", folder.uri);
            self.add(&folder, &mut workspaces, client.clone());
        }
    }

    #[allow(clippy::significant_drop_tightening)]
    pub fn changed(&self, changes: DidChangeWorkspaceFoldersParams, client: &Client) {
        let mut workspaces = self
            .workspaces
            .write()
            .expect("Failed to lock workspaces for writing");
        for removed in changes.event.removed {
            if workspaces.contains_key(&removed.uri) {
                workspaces.remove(&removed.uri);
            }
        }
        for added in changes.event.added {
            if workspaces.contains_key(&added.uri) {
                workspaces.remove(&added.uri);
            }
            self.add(&added, &mut workspaces, client.clone());
        }
    }

    pub fn guess_workspace(&self, uri: &Url) -> Option<EditorWorkspace> {
        let mut best = None;
        let mut best_len = 0;
        for (folder, workspace) in self
            .workspaces
            .read()
            .expect("Failed to lock workspaces for reading")
            .iter()
        {
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
            }
            tries -= 1;
            if tries == 0 {
                return None;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }

    #[allow(clippy::unused_self)]
    fn add(
        &self,
        added: &WorkspaceFolder,
        workspaces: &mut HashMap<Url, EditorWorkspace>,
        client: Client,
    ) {
        debug!("adding workspace {}", added.uri);
        if let Some(workspace) = EditorWorkspace::new(added) {
            workspaces.insert(added.uri.clone(), workspace.clone());
            let config_workspace = workspace.clone();
            let config_client = client.clone();
            tokio::spawn(async move {
                ConfigAnalyzer::get().workspace_added(&config_workspace, config_client);
            });
            tokio::spawn(async move {
                SqfAnalyzer::get().workspace_added(&workspace, client);
            });
        } else {
            debug!("failed to add workspace {}", added.uri);
        }
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct EditorWorkspace {
    root: PathBuf,
    url: Url,
    workspace: WorkspacePath,
}

impl EditorWorkspace {
    pub fn new(folder: &WorkspaceFolder) -> Option<Self> {
        if folder.uri.scheme() == "file" {
            let root = PathBuf::from(
                urlencoding::decode(
                    folder
                        .uri
                        .to_string()
                        .replace(if cfg!(windows) { "file:///" } else { "file://" }, "")
                        .as_str(),
                )
                .expect("Failed to decode URL")
                .to_string(),
            );
            let mut builder = Workspace::builder().physical(&root, LayerType::Source);
            let include = root.join("include");
            if include.is_dir() {
                builder = builder.physical(&include, LayerType::Include);
            }
            let Ok(workspace) = builder.finish(None, true, &PDriveOption::Disallow) else {
                return None;
            };
            Some(Self {
                workspace,
                url: folder.uri.clone(),
                root,
            })
        } else {
            None
        }
    }

    pub fn join_url(&self, url: &Url) -> Result<WorkspacePath, String> {
        let decoded_path = urlencoding::decode(url.path()).map_err(|e| format!("{e}"))?;
        let workspace_path = urlencoding::decode(self.url.path()).map_err(|e| format!("{e}"))?;
        let Some(path) = decoded_path.strip_prefix(workspace_path.as_ref()) else {
            return Err("URL is not in workspace".to_string());
        };
        self.workspace.join(path).map_err(|e| format!("{e}"))
    }

    pub fn to_url(&self, path: &WorkspacePath) -> Url {
        let include = path.is_include();
        // trim the workspace path
        let path = path
            .as_str()
            .strip_prefix(self.workspace.as_str())
            .expect("Failed to strip workspace prefix from path");
        let path = path.replace('\\', "/");
        // url encode the path
        let path = urlencoding::encode(&path);
        let path = path.replace("%2F", "/");
        let mut url = self.url.clone();
        url.set_path(
            format!(
                "{}{}{}",
                url.path(),
                if include { "/include" } else { "" },
                path
            )
            .as_str(),
        );
        url
    }

    pub const fn root(&self) -> &WorkspacePath {
        &self.workspace
    }

    pub const fn root_disk(&self) -> &PathBuf {
        &self.root
    }

    pub fn config(&self) -> Option<ProjectConfig> {
        let path = self.root.join(".hemtt").join("project.toml");
        if path.is_file() {
            match ProjectConfig::from_file(&path) {
                Ok(config) => Some(config),
                Err(e) => {
                    debug!("failed to load config: {:?}", e);
                    None
                }
            }
        } else {
            debug!("no config found at {:?}", path);
            None
        }
    }

    #[allow(dead_code)]
    pub const fn url(&self) -> &Url {
        &self.url
    }
}
