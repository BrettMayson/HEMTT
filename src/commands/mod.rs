mod init;
pub use init::Init;

mod template;
pub use template::Template;

pub mod build;
pub use build::Build;

mod pack;
pub use pack::Pack;

mod clean;
pub use clean::Clean;

mod status;
pub use status::Status;

mod update;
pub use update::Update;

use crate::{HEMTTError, Project};

pub trait Command {
    // (name, description)
    fn register(&self) -> clap::App;
    fn run(&self, _args: &clap::ArgMatches, _project: Project) -> Result<(), HEMTTError> {
        unimplemented!();
    }
    fn require_project(&self) -> bool {
        true
    }
    fn run_no_project(&self, _args: &clap::ArgMatches) -> Result<(), HEMTTError> {
        unimplemented!();
    }
}
