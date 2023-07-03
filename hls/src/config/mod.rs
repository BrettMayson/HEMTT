use dashmap::DashMap;
use hemtt::context::Context;
use hemtt_preprocessor::{preprocess_file, Resolver};
use tower_lsp::{
    lsp_types::{
        DidChangeWorkspaceFoldersParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    },
    Client,
};
use tracing::{debug, error, info, warn};
use url::Url;
use vfs::VfsPath;

mod diagnostics;

use self::diagnostics::Diagnostics;

#[derive(Debug)]
pub struct ConfigServer {
    client: Client,
    workspaces: DashMap<Url, Context>,
    diagnostics: Diagnostics,
}

impl ConfigServer {
    pub fn new(client: Client) -> Self {
        Self {
            diagnostics: Diagnostics::new(client.clone()),
            client,
            workspaces: DashMap::new(),
        }
    }

    pub async fn initialize(&self) {
        let folders = self.client.workspace_folders().await;
        if let Ok(Some(folders)) = folders {
            for folder in folders {
                self.add_workspace(folder.uri).await;
            }
        }
    }

    pub async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        for removed in params.event.removed {
            self.remove_workspace(removed.uri).await;
        }
        for added in params.event.added {
            self.add_workspace(added.uri).await;
        }
    }

    pub async fn did_open(&self, params: DidOpenTextDocumentParams) {
        if let Some((workspace, file)) = self.in_workspace(params.text_document.uri).await {
            if let Some(addon) = self.find_addon(file).await {
                self.check_addon((&workspace.0, &workspace.1), &addon).await;
            }
        }
    }

    pub async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some((workspace, file)) = self.in_workspace(params.text_document.uri).await {
            if let Some(addon) = self.find_addon(file).await {
                self.check_addon((&workspace.0, &workspace.1), &addon).await;
            }
        }
    }

    async fn add_workspace(&self, mut uri: Url) {
        uri.set_path(&format!("{}/", uri.path()));
        if let Ok(path) = uri.to_file_path() {
            match Context::new(path, "ls") {
                Ok(context) => {
                    let addons = context.addons().to_vec();
                    for addon in addons {
                        self.check_addon(
                            (&uri, &context),
                            &context.vfs().join(addon.folder()).unwrap(),
                        )
                        .await;
                    }
                    info!("Added workspace: {}", &uri);
                    self.workspaces.insert(uri, context);
                }
                Err(e) => {
                    error!("Failed to load workspace: {}", e)
                }
            }
        } else {
            error!("Failed to convert workspace uri to path: {}", uri)
        }
    }

    async fn remove_workspace(&self, uri: Url) {
        info!("Removed workspace: {}", &uri);
        self.workspaces.remove(&uri);
    }

    async fn in_workspace(&self, uri: Url) -> Option<((Url, Context), VfsPath)> {
        for workspace in &self.workspaces {
            let url_string = workspace.key().to_string();
            if uri.to_string().starts_with(&url_string) {
                return Some((
                    (workspace.key().clone(), workspace.clone()),
                    workspace
                        .vfs()
                        .join(uri.to_string().trim_start_matches(&url_string))
                        .unwrap(),
                ));
            }
        }
        None
    }

    async fn find_addon(&self, file: VfsPath) -> Option<VfsPath> {
        let mut max = 50;
        let mut file = file;
        loop {
            max -= 1;
            if max == 0 {
                return None;
            }
            let parent = file.parent();
            if parent.is_root() {
                return None;
            }
            if ["addons", "optionals"].contains(&parent.filename().as_str()) {
                return Some(file);
            }
            file = parent;
        }
    }

    pub async fn check_addon(&self, workspace: (&Url, &Context), addon: &VfsPath) {
        let Ok(config) = addon.join("config.cpp") else {
            return;
        };
        if let Ok(false) | Err(_) = config.exists() {
            debug!("config.cpp does not exist");
            return;
        }
        let resolver = Resolver::new(workspace.1.vfs(), workspace.1.prefixes());
        match preprocess_file(&config, &resolver) {
            Ok(processed) => match hemtt_config::parse(&processed) {
                Ok(report) => {
                    let mut diags = Vec::new();
                    for error in report.errors() {
                        diags.extend(error.generate_processed_lsp(&processed));
                        if let Some(diag) = error.generate_lsp() {
                            diags.push(diag)
                        }
                    }
                    self.diagnostics
                        .update_addon(workspace.0, addon, diags)
                        .await;
                }
                Err(e) => {
                    warn!("todo err to lsp");
                    error!("{}", e.join("\n"))
                }
            },
            Err(err) => {
                if let hemtt_preprocessor::Error::Code(code) = err {
                    if let Some(diag) = code.generate_lsp() {
                        self.diagnostics
                            .update_addon(workspace.0, addon, vec![diag])
                            .await;
                    } else {
                        error!(
                            "config::check_file: error not ready for lsp {:#?}",
                            code.ident()
                        )
                    }
                } else {
                    error!("config::check_file: error not ready for lsp {:#?}", err)
                }
            }
        }
    }
}
