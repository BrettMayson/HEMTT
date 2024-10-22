//! A workspace (directory) containing addons and / or missions

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use hemtt_common::{
    config::{PDriveOption, ProjectConfig},
    prefix::{Prefix, FILES},
};
use pdrive::PDrive;
use tracing::trace;
use vfs::{AltrootFS, MemoryFS, OverlayFS, PhysicalFS, VfsPath};

// Re-export for macros
pub use linkme;
pub use paste;

pub mod addons;
pub mod error;
pub mod lint;
pub mod path;
pub mod pdrive;
pub mod position;
pub mod reporting;

use pdrive::search as pdrive_search;

pub use error::Error;
#[allow(clippy::module_name_repetitions)]
pub use path::WorkspacePath;

#[derive(Debug, PartialEq, Eq)]
/// A workspace (directory) containing addons and / or missions
pub struct Workspace {
    pub(crate) vfs: VfsPath,
    pub(crate) layers: Vec<(VfsPath, LayerType)>,
    pub(crate) project: Option<ProjectConfig>,
    pub(crate) pointers: HashMap<String, VfsPath>,
    pub(crate) addons: Vec<VfsPath>,
    pub(crate) missions: Vec<VfsPath>,
    pub(crate) pdrive: Option<PDrive>,
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

    #[must_use]
    /// Returns the pdrive
    pub const fn pdrive(&self) -> Option<&PDrive> {
        self.pdrive.as_ref()
    }

    /// Create a new workspace from a vfs path
    ///
    /// # Errors
    /// [`Error::Vfs`] if the workspace could not be created
    pub fn create(
        vfs: VfsPath,
        layers: Vec<(VfsPath, LayerType)>,
        project: Option<ProjectConfig>,
        discovery: bool,
        pdrive: &PDriveOption,
    ) -> Result<WorkspacePath, Error> {
        let mut workspace = Self {
            vfs,
            layers,
            project,
            pointers: HashMap::new(),
            addons: Vec::new(),
            missions: Vec::new(),
            pdrive: if pdrive == &PDriveOption::Require {
                pdrive_search()
            } else {
                None
            },
        };
        if discovery {
            workspace.discover()?;
        }
        Ok(WorkspacePath {
            data: Arc::new(path::WorkspacePathData {
                path: workspace.vfs.root(),
                workspace: Arc::new(workspace),
            }),
        })
    }

    fn discover(&mut self) -> Result<(), Error> {
        for entry in self.vfs.walk_dir()? {
            let Ok(entry) = entry else {
                trace!("unknown issue with entry: {:?}", entry);
                continue;
            };
            if entry.as_str().contains(".hemtt") {
                continue;
            }
            match entry.filename().to_lowercase().as_str() {
                "config.cpp" => {
                    trace!("config.cpp: {:?}", entry);
                    self.addons.push(entry);
                }
                "mission.sqm" => {
                    trace!("mission.sqm: {:?}", entry);
                    self.missions.push(entry);
                }
                _ => {
                    if FILES.contains(&entry.filename().to_lowercase().as_str()) {
                        trace!("Prefix: {:?}", entry);
                        let prefix = Prefix::new(&entry.read_to_string()?)?;
                        self.pointers.insert(
                            format!("/{}", prefix.to_string().to_lowercase().replace('\\', "/")),
                            entry.parent(),
                        );
                    }
                }
            }
        }
        Ok(())
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
    pub fn finish(
        self,
        project: Option<ProjectConfig>,
        discovery: bool,
        pdrive: &PDriveOption,
    ) -> Result<WorkspacePath, Error> {
        let mut layers = self.layers.clone();
        layers.reverse();
        Workspace::create(
            OverlayFS::new(&layers.into_iter().map(|(l, _)| l).collect::<Vec<_>>()).into(),
            self.layers,
            project,
            discovery,
            pdrive,
        )
    }
}
