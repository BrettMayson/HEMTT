use hemtt_project::Project;

pub trait Command {
    // (name, description)
    fn register(&self) -> clap::App;
    fn run(&self, _args: &clap::ArgMatches, _project: Project) -> Result<(), ()> {
        unimplemented!();
    }
    fn require_project(&self) -> bool {
        true
    }
    fn run_no_project(&self, _args: &clap::ArgMatches) -> Result<(), ()> {
        unimplemented!();
    }
    fn can_announce(&self) -> bool {
        true
    }
}
