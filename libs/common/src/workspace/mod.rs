use std::{collections::HashMap, path::PathBuf, sync::Arc};

use tracing::trace;
use vfs::{AltrootFS, MemoryFS, OverlayFS, PhysicalFS, VfsPath};

mod error;
mod path;

pub use error::Error;
pub use path::WorkspacePath;

use crate::prefix::{Prefix, FILES};

pub struct Workspace {
    pub(crate) vfs: VfsPath,
    pub(crate) pointers: HashMap<PathBuf, VfsPath>,
    pub(crate) addons: Vec<VfsPath>,
    pub(crate) missions: Vec<VfsPath>,
}

impl Workspace {
    pub fn builder() -> WorkspaceBuilder {
        WorkspaceBuilder::default()
    }

    pub fn create(vfs: VfsPath) -> Result<WorkspacePath, Error> {
        let mut workspace = Self {
            vfs,
            pointers: HashMap::new(),
            addons: Vec::new(),
            missions: Vec::new(),
        };
        for entry in workspace.vfs.walk_dir()? {
            let entry = entry?;
            if entry.is_dir()? {
                continue;
            }
            match entry.filename().to_lowercase().as_str() {
                "config.cpp" => {
                    trace!("config.cpp: {:?}", entry);
                    workspace.addons.push(entry);
                }
                "mission.sqm" => {
                    trace!("mission.sqm: {:?}", entry);
                    workspace.missions.push(entry);
                }
                _ => {
                    if FILES.contains(&entry.filename().to_lowercase().as_str()) {
                        trace!("Prefix: {:?}", entry);
                        let prefix = Prefix::new(&entry.read_to_string()?)?;
                        workspace
                            .pointers
                            .insert(prefix.as_pathbuf(), entry.parent());
                    }
                }
            }
        }
        Ok(WorkspacePath {
            path: workspace.vfs.root(),
            workspace: Arc::new(workspace),
        })
    }
}

#[derive(Default)]
pub struct WorkspaceBuilder {
    layers: Vec<VfsPath>,
}

impl WorkspaceBuilder {
    pub fn physical(mut self, path: &PathBuf) -> Self {
        self.layers
            .push(AltrootFS::new(PhysicalFS::new(path).into()).into());
        self
    }

    pub fn memory(mut self) -> Self {
        self.layers.push(MemoryFS::new().into());
        self
    }

    pub fn finish(self) -> Result<WorkspacePath, Error> {
        Workspace::create(OverlayFS::new(&self.layers).into())
    }
}
