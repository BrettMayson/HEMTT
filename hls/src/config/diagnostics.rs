use std::{collections::HashMap, sync::Arc};

use dashmap::DashMap;
use tokio::sync::RwLock;
use tower_lsp::{lsp_types::Diagnostic, Client};
use tracing::debug;
use url::Url;
use vfs::VfsPath;

#[derive(Debug)]
pub struct Diagnostics {
    client: Client,
    addons: DashMap<Url, Addon>,
}

impl Diagnostics {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            addons: DashMap::new(),
        }
    }

    pub async fn update_addon(
        &self,
        workspace: &Url,
        addon: &VfsPath,
        codes: Vec<(VfsPath, Diagnostic)>,
    ) {
        let addon_url = workspace
            .join(addon.as_str().trim_start_matches('/'))
            .unwrap();
        debug!("Updating addon: {}", addon_url);
        self.addons
            .entry(addon_url.clone())
            .or_insert_with(|| Addon {
                client: self.client.clone(),
                root: addon_url,
                last_files: Arc::new(RwLock::new(Vec::new())),
            })
            .update(codes)
            .await;
    }
}

#[derive(Debug)]
struct Addon {
    client: Client,
    root: Url,
    last_files: Arc<RwLock<Vec<String>>>,
}

impl Addon {
    pub async fn update(&self, codes: Vec<(VfsPath, Diagnostic)>) {
        let mut map = HashMap::new();
        for (path, code) in codes {
            let path = self
                .root
                .join(path.as_str().trim_start_matches("/addons/"))
                .unwrap();
            map.entry(path).or_insert_with(Vec::new).push(code);
        }
        let new_files = map.keys().map(|k| k.to_string()).collect::<Vec<_>>();
        for last_file in self.last_files.read().await.iter() {
            debug!("clearing diagnostics for {}", last_file);
            if !new_files.contains(last_file) {
                self.client
                    .publish_diagnostics(Url::parse(last_file).unwrap(), Vec::new(), None)
                    .await;
            }
        }
        *self.last_files.write().await = new_files;
        for files in map {
            self.client
                .publish_diagnostics(files.0, files.1, None)
                .await;
        }
    }
}
