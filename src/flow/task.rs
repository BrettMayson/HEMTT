#[cfg(not(windows))]
use indicatif::ProgressBar;
#[cfg(windows)]
use indicatif_windows::ProgressBar;

use crate::{Addon, AddonList, HEMTTError, Project, Report, Stage};

// A task is an independent item to be ran
pub trait Task: objekt::Clone + std::marker::Send + std::marker::Sync {
    fn can_run(&self, _: &Addon, _: &Report, _: &Project, _: &Stage) -> Result<bool, HEMTTError> {
        Ok(false)
    }
    fn parallel(&self, _: &Addon, _: &Report, _: &Project, _: &Stage, _: &ProgressBar) -> Result<Report, HEMTTError> {
        unimplemented!()
    }
    fn single(&self, _: Vec<Result<(Report, Addon), HEMTTError>>, _: &Project, _: &Stage) -> AddonList {
        unimplemented!()
    }
}
objekt::clone_trait_object!(Task);
