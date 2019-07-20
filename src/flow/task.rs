use indicatif::ProgressBar;

use crate::{Addon, AddonList, HEMTTError, Project, Report};

// A task is an independent item to be ran
pub trait Task: objekt::Clone + std::marker::Send + std::marker::Sync {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project) -> Result<bool, HEMTTError> { Ok(false) }
    fn parallel(&self, _: &Addon, _: &Report, _: &Project, _: &ProgressBar) -> Result<Report, HEMTTError> { unimplemented!() }
    fn single(&self, _: Vec<Result<(Report, Addon), HEMTTError>>, _: &Project) -> AddonList { unimplemented!() }
}
objekt::clone_trait_object!(Task);
