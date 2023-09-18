//! A workspace (directory) containing addons and / or missions

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use tracing::trace;
use vfs::{AltrootFS, MemoryFS, OverlayFS, PhysicalFS, VfsPath};

mod error;
mod path;

pub use error::Error;
pub use path::WorkspacePath;

use crate::prefix::{Prefix, FILES};

#[derive(Debug, PartialEq, Eq)]
/// A workspace (directory) containing addons and / or missions
pub struct Workspace {
    pub(crate) vfs: VfsPath,
    pub(crate) pointers: HashMap<PathBuf, VfsPath>,
    pub(crate) addons: Vec<VfsPath>,
    pub(crate) missions: Vec<VfsPath>,
}

impl Workspace {
    #[must_use]
    /// Create a new workspace builder
    pub fn builder() -> WorkspaceBuilder {
        WorkspaceBuilder::default()
    }

    /// Create a new workspace from a vfs path
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
/// A workspace builder
pub struct WorkspaceBuilder {
    layers: Vec<VfsPath>,
}

impl WorkspaceBuilder {
    #[must_use]
    /// Add a physical layer to the virtual filesystem
    pub fn physical(mut self, path: &PathBuf) -> Self {
        self.layers
            .push(AltrootFS::new(PhysicalFS::new(path).into()).into());
        self
    }

    #[must_use]
    /// Add a memory layer to the virtual filesystem
    pub fn memory(mut self) -> Self {
        self.layers.push(MemoryFS::new().into());
        self
    }

    /// Finish building the workspace
    pub fn finish(self) -> Result<WorkspacePath, Error> {
        let mut layers = self.layers;
        layers.reverse();
        Workspace::create(OverlayFS::new(&layers).into())
    }
}
