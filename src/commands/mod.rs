mod init;
pub use init::Init;

mod template;
pub use template::Template;

mod build;
pub use build::Build;

use crate::project::Project;
use crate::HEMTTError;

pub trait Command {
    // (name, description)
    fn register(&self) -> (&str, clap::App);
    fn run(&self, _args: &clap::ArgMatches, _project: Project) -> Result<(), HEMTTError> {
        unimplemented!();
    }
    fn require_project(&self) -> bool { true }
    fn run_no_project(&self, _args: &clap::ArgMatches) -> Result<(), HEMTTError> {
        unimplemented!();
    }
}
