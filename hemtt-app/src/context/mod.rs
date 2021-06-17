use std::{path::PathBuf, sync::RwLock};

use state::Container;
use vfs::{
    impls::{altroot::AltrootFS, overlay::OverlayFS},
    MemoryFS, PhysicalFS, VfsPath,
};

use crate::Project;
use hemtt::{Addon, HEMTTError};

mod addon;
pub use addon::{AddonContext, AddonListContext};

pub struct Context<'a> {
    project: &'a Project,
    task_pad: usize,
    fs: VfsPath,
    root: PathBuf,
    // stage: &Stage,
    message_info: RwLock<(String, String)>,
    pub container: Container![Send + Sync],
}

impl<'a> Context<'a> {
    pub fn new(project: &'a Project) -> Result<Self, HEMTTError> {
        let root = Project::find_root()?;
        Ok(Self {
            project,
            task_pad: 0usize,
            fs: AltrootFS::new(
                OverlayFS::new(&[
                    MemoryFS::new().into(),
                    AltrootFS::new(PhysicalFS::new(root.clone()).into()).into(),
                ])
                .into(),
            )
            .into(),
            root,

            message_info: RwLock::new((String::from("internal init"), String::from("new"))),
            container: <Container![Send + Sync]>::new(),
        })
    }

    pub fn project(&self) -> &Project {
        self.project
    }

    pub fn fs(&self) -> &VfsPath {
        &self.fs
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn task_pad(&self) -> usize {
        self.task_pad
    }

    pub fn set_task_pad(&mut self, pad: usize) {
        self.task_pad = pad
    }

    pub fn set_message_info(&self, stage: String, task: String) {
        *self.message_info.write().unwrap() = (stage, task);
    }
}

impl<'a, 'b> Context<'a> {
    // pub fn get_single(&'a self, addon: &'b Addon) -> Result<AddonContext<'a, 'b>, HEMTTError> {
    //     AddonContext::new(&self, &addon)
    // }
    pub fn get_list(
        &'a mut self,
        addons: Vec<Addon>,
    ) -> Result<AddonListContext<'a, 'b>, HEMTTError> {
        AddonListContext::new(self, addons)
    }
}
