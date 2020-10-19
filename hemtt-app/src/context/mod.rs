use crate::{AddonList, Project};
use hemtt::Addon;
use hemtt_cache::FileCache;

mod addon;
pub use addon::{AddonContext, AddonListContext};

pub struct Context<'a> {
    pub project: &'a Project,
    pub cache: FileCache,
    pub task_pad: usize,
    // stage: &Stage,
}

impl<'a> Context<'a> {
    pub fn new(project: &'a Project) -> Self {
        Self {
            project,
            cache: FileCache::new(),
            task_pad: 0usize,
        }
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
