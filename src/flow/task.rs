use crate::{Addon, AddonList, HEMTTError, Project, Report, Stage};

// A task is an independent item to be ran
pub trait Task: dyn_clone::DynClone + std::marker::Send + std::marker::Sync {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(false)
    }
    fn parallel(&self, _: &Addon, _: &Report, _: &Project, _: &Stage) -> Result<Report, HEMTTError> {
        unimplemented!()
    }
    fn single(&self, _: Vec<Result<(Report, Addon), HEMTTError>>, _: &Project, _: &Stage) -> AddonList {
        unimplemented!()
    }
}
dyn_clone::clone_trait_object!(Task);
