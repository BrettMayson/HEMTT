//! A workspace (directory) containing addons and / or missions

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use tracing::trace;
use vfs::{AltrootFS, MemoryFS, OverlayFS, PhysicalFS, VfsPath};

mod error;
mod path;

pub use error::Error;
#[allow(clippy::module_name_repetitions)]
pub use path::WorkspacePath;

use crate::{
    prefix::{Prefix, FILES},
    project::ProjectConfig,
};

#[derive(Debug, PartialEq, Eq)]
/// A workspace (directory) containing addons and / or missions
pub struct Workspace {
    pub(crate) vfs: VfsPath,
    pub(crate) layers: Vec<(VfsPath, LayerType)>,
    pub(crate) project: Option<ProjectConfig>,
    pub(crate) pointers: HashMap<String, VfsPath>,
    pub(crate) addons: Vec<VfsPath>,
    pub(crate) missions: Vec<VfsPath>,
}

impl Workspace {
    #[must_use]
    /// Create a new workspace builder
    pub fn builder() -> WorkspaceBuilder {
        WorkspaceBuilder::default()
    }

    #[must_use]
    /// Returns the project config
    pub const fn project(&self) -> Option<&ProjectConfig> {
        self.project.as_ref()
    }

    /// Create a new workspace from a vfs path
    ///
    /// # Errors
    /// [`Error::Vfs`] if the workspace could not be created
    pub fn create(
        vfs: VfsPath,
        layers: Vec<(VfsPath, LayerType)>,
        project: Option<ProjectConfig>,
    ) -> Result<WorkspacePath, Error> {
        let mut workspace = Self {
            vfs,
            layers,
            project,
            pointers: HashMap::new(),
            addons: Vec::new(),
            missions: Vec::new(),
        };
        for entry in workspace.vfs.walk_dir()? {
            let Ok(entry) = entry else {
                trace!("unknown issue with entry: {:?}", entry);
                continue;
            };
            if entry.is_dir()? {
                continue;
            }
            if entry.as_str().contains(".hemtt") {
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
                        workspace.pointers.insert(
                            format!("/{}", prefix.to_string().replace('\\', "/")),
                            entry.parent(),
                        );
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayerType {
    Source,
    Include,
    Build,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Default)]
/// A workspace builder
pub struct WorkspaceBuilder {
    layers: Vec<(VfsPath, LayerType)>,
}

impl WorkspaceBuilder {
    #[must_use]
    /// Add a physical layer to the virtual filesystem
    pub fn physical(mut self, path: &PathBuf, layer_type: LayerType) -> Self {
        self.layers.push((
            AltrootFS::new(PhysicalFS::new(path).into()).into(),
            layer_type,
        ));
        self
    }

    #[must_use]
    /// Add a memory layer to the virtual filesystem
    pub fn memory(mut self) -> Self {
        self.layers.push((MemoryFS::new().into(), LayerType::Build));
        self
    }

    /// Finish building the workspace
    ///
    /// # Errors
    /// [`Error::Vfs`] if the workspace could not be built
    pub fn finish(self, project: Option<ProjectConfig>) -> Result<WorkspacePath, Error> {
        let mut layers = self.layers.clone();
        layers.reverse();
        Workspace::create(
            OverlayFS::new(&layers.into_iter().map(|(l, _)| l).collect::<Vec<_>>()).into(),
            self.layers,
            project,
        )
    }
}
