use vfs::{MemoryFS, PhysicalFS, VfsPath, impls::overlay::OverlayFS};

use crate::{AddonList, Project};
use hemtt::{Addon, HEMTTError};

mod addon;
pub use addon::{AddonContext, AddonListContext};

pub struct Context<'a> {
    project: &'a Project,
    pub task_pad: usize,
    fs: VfsPath,
    // stage: &Stage,
}

impl<'a> Context<'a> {
    pub fn new(project: &'a Project) -> Result<Self, HEMTTError> {
        Ok(Self {
            project,
            task_pad: 0usize,
            fs: OverlayFS::new(&[
                MemoryFS::new().into(),
                PhysicalFS::new(Project::find_root()?).into(),
            ]).into(),
        })
    }

    pub fn project(&self) -> &Project {
        self.project
    }

    pub fn fs(&self) -> &VfsPath {
        &self.fs
    }
}

impl<'a, 'b> Context<'a> {
    pub fn get_single(&'a self, addon: &'b Addon) -> AddonContext<'a, 'b> {
        AddonContext {
            global: &self,
            addon: &addon,
        }
    }
    pub fn get_list(&'a self, addons: &'b mut AddonList) -> AddonListContext<'a, 'b> {
        AddonListContext {
            global: &self,
            addons,
        }
    }
}
