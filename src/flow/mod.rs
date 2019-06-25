mod report;
pub use report::Report;

use crate::error::HEMTTError;
use crate::build::Addon;
use crate::project::Project;

// A flow is a queue of tasks to run a various points during the app cycle
pub struct Flow {
    pub checks: Vec<Box<dyn Task>>,
    pub pre_build: Vec<Box<dyn Task>>,
    pub post_build: Vec<Box<dyn Task>>,
    pub release: Vec<Box<dyn Task>>,
}

impl Flow {
    pub fn execute(&self, addons: Vec<Addon>, p: &Project) -> Result<Report, HEMTTError> {
        for addon in addons {
            for check in &self.checks {
                if check.chk_can_run(&addon, p)? {
                    let report = check.chk_run(&addon, p)?;
                    report.display();
                }
            }
        }
        Ok(Report::new())
    }
}

// A task is an independent item to be ran
pub trait Task {
    fn chk_can_run(&self, _addon: &crate::build::Addon, _p: &Project) -> Result<bool, HEMTTError> { Ok(false) }
    fn chk_run(&self, _addon: &crate::build::Addon, _p: &Project) -> Result<Report, HEMTTError> { unimplemented!() }
    fn pre_can_run(&self, _addon: &crate::build::Addon, _p: &Project) -> Result<bool, HEMTTError> { Ok(false) }
    fn pre_run(&self, _addon: &crate::build::Addon, _p: &Project) -> Result<Report, HEMTTError> { unimplemented!() }
    fn post_can_run(&self, _addon: &crate::build::Addon, _p: &Project) -> Result<bool, HEMTTError> { Ok(false) }
    fn post_run(&self, _addon: &crate::build::Addon, _p: &Project) -> Result<Report, HEMTTError> { unimplemented!() }
    fn rel_can_run(&self, _addon: &crate::build::Addon, _p: &Project) -> Result<bool, HEMTTError> { Ok(false) }
    fn rel_run(&self, _addon: &crate::build::Addon, _p: &Project) -> Result<Report, HEMTTError> { unimplemented!() }
}
