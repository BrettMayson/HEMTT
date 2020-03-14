mod init;
pub use init::Init;

mod template;
pub use template::Template;

pub mod build;
pub use build::Build;

pub mod bug;
pub use bug::Bug;

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
    fn can_announce(&self) -> bool {
        true
    }
}

pub fn building_args<'a, 'b>() -> Vec<clap::Arg<'a, 'b>> {
    vec![
        clap::Arg::with_name("addons")
            .help("Addons to build")
            .takes_value(true)
            .multiple(true)
            .required(false),
        clap::Arg::with_name("release")
            .help("Build a release")
            .long("release")
            .conflicts_with("dev"),
        clap::Arg::with_name("force")
            .help("Rebuild existing files")
            .long("force")
            .short("f"),
        clap::Arg::with_name("force-release")
            .help("Remove an existing release")
            .long("force-release"),
        clap::Arg::with_name("skip")
            .help("Skip addons")
            .long("skip")
            .takes_value(true),
        clap::Arg::with_name("opts")
            .help("Only build listed optional addons")
            .long("opts")
            .short("o")
            .takes_value(true)
            .multiple(true),
        clap::Arg::with_name("compats")
            .help("Only build listed compat addons")
            .long("compats")
            .short("c")
            .takes_value(true)
            .multiple(true),
    ]
}
