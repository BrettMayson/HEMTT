use crate::{Addon, AddonList, HEMTTError, OkSkip, Project, Stage};

// A task is an independent item to be ran
pub trait Task: dyn_clone::DynClone + std::marker::Send + std::marker::Sync {
    fn can_run(&self, _: &Addon, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(false)
    }
    fn parallel(&self, _: &Addon, _: &Project, _: &Stage) -> Result<OkSkip, HEMTTError> {
        unimplemented!()
    }
    fn single(&self, _: AddonList, _: &Project, _: &Stage) -> Result<AddonList, HEMTTError> {
        unimplemented!()
    }
}
dyn_clone::clone_trait_object!(Task);
