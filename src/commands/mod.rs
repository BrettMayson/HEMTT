mod init;
pub use init::Init;

mod template;
pub use template::Template;

use crate::project::Project;

pub trait Command {
    // (name, description)
    fn register(&self) -> (&str, clap::App);
    fn run(&self, _args: &clap::ArgMatches, _project: Project) -> bool {
        unimplemented!();
    }
    fn require_project(&self) -> bool { true }
    fn run_no_project(&self, _args: &clap::ArgMatches) -> bool {
        unimplemented!();
    }
}
