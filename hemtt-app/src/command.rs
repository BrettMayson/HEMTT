use hemtt::{HEMTTError, Project};

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
    fn can_announce(&self) -> bool {
        true
    }
}
