use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use clap::App;

#[macro_use]
pub mod macros;

pub mod commands;
pub mod error;
pub mod files;
pub mod flow;
pub mod project;
pub mod render;
mod startup;
pub mod utilities;

pub use build::addon::{Addon, AddonLocation};
pub use commands::{build, Command};
pub use error::{FileErrorLineNumber, HEMTTError, IOPathError};
pub use files::{FileCache, RenderedFiles};
pub use flow::{BuildScript, Flow, Report, Stage, Step, Task};
pub use project::Project;

pub type AddonList = Result<Vec<Result<(Report, Addon), HEMTTError>>, HEMTTError>;

lazy_static::lazy_static! {
    pub static ref CACHED: Arc<Mutex<FileCache>> = Arc::new(Mutex::new(FileCache::new()));
    pub static ref RENDERED: Arc<Mutex<RenderedFiles>> = Arc::new(Mutex::new(RenderedFiles::new()));
    pub static ref REPORTS: Arc<Mutex<HashMap<String, Report>>> = Arc::new(Mutex::new(HashMap::new()));

    pub static ref CI: bool = std::env::args().any(|x| x == "--ci") || is_ci();

    pub static ref VERSION: &'static str = {
        let mut version = env!("CARGO_PKG_VERSION").to_string();
        if let Some(v) = option_env!("GIT_HASH") {
            version.push_str("-");
            version.push_str(v);
        }
        if cfg!(debug_assertions) {
            version.push_str("-debug");
        }
        Box::leak(Box::new(version))
    };
}

pub fn is_ci() -> bool {
    // TODO: replace with crate if a decent one comes along
    let checks = vec![
        "CI",
        "APPVEYOR",
        "SYSTEM_TEAMFOUNDATIONCOLLECTIONURI",
        "bamboo_planKey",
        "BITBUCKET_COMMIT",
        "BITRISE_IO",
        "BUDDY_WORKSPACE_ID",
        "BUILDKITE",
        "CIRCLECI",
        "CIRRUS_CI",
        "CODEBUILD_BUILD_ARN",
        "DRONE",
        "DSARI",
        "GITLAB_CI",
        "GO_PIPELINE_LABEL",
        "HUDSON_URL",
        "MAGNUM",
        "NETLIFY_BUILD_BASE",
        "PULL_REQUEST",
        "NEVERCODE",
        "SAILCI",
        "SEMAPHORE",
        "SHIPPABLE",
        "TDDIUM",
        "STRIDER",
        "TEAMCITY_VERSION",
        "TRAVIS",
    ];
    for check in checks {
        if std::env::var(check).is_ok() {
            return true;
        }
    }
    false
}

pub fn execute(input: &[String], root: bool) -> Result<(), HEMTTError> {
    let mut app = App::new("HEMTT")
        .version(*crate::VERSION)
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            clap::Arg::with_name("jobs")
                .global(true)
                .help("Number of parallel jobs, defaults to # of CPUs")
                .takes_value(true)
                .long("jobs")
                .short("j"),
        )
        .arg(
            clap::Arg::with_name("debug")
                .global(true)
                .help("Turn debugging information on")
                .long("debug")
                .short("d"),
        )
        .arg(
            clap::Arg::with_name("time")
                .global(true)
                .help("Time the execution")
                .long("time"),
        )
        .arg(
            clap::Arg::with_name("ci") // This is not actually checked by clap, see lib.rs
                .global(true)
                .help("Run in CI mode")
                .long("ci"),
        );

    let mut commands: Vec<Box<dyn Command>> = Vec::new();
    let mut hash_commands: HashMap<String, &Box<dyn Command>> = HashMap::new();

    // Add commands here
    commands.push(Box::new(commands::Init {}));
    commands.push(Box::new(commands::Template {}));
    commands.push(Box::new(commands::Build {}));
    commands.push(Box::new(commands::Pack {}));
    commands.push(Box::new(commands::Clean {}));
    commands.push(Box::new(commands::Status {}));
    commands.push(Box::new(commands::Update {}));

    // Add utilities here
    commands.push(Box::new(utilities::Translation {}));
    commands.push(Box::new(utilities::MissionGenerate {}));
    commands.push(Box::new(utilities::Zip {}));
    // Windows only utilities
    #[cfg(windows)]
    {
        commands.push(Box::new(utilities::FilePatching {}));
    }

    for command in commands.iter() {
        let sub = command.register();
        hash_commands.insert(sub.get_name().to_owned(), command);
        app = app.subcommand(sub);
    }

    let matches = app.get_matches_from(input);

    let start = if matches.is_present("time") {
        Some(Instant::now())
    } else {
        None
    };

    if root {
        rayon::ThreadPoolBuilder::new()
            .num_threads(if let Some(jobs) = matches.value_of("jobs") {
                usize::from_str_radix(jobs, 10)?
            } else {
                num_cpus::get()
            })
            .build_global()
            .unwrap();
    }

    match matches.subcommand_name() {
        Some(v) => match hash_commands.get(v) {
            Some(c) => {
                let sub_matches = matches.subcommand_matches(v).unwrap();
                if c.require_project() {
                    let project = Project::read()?;
                    if root {
                        println!("HEMTT {}", *crate::VERSION);
                        println!("Environment: {}", project::environment());
                        println!();
                        startup::startup();
                    }
                    c.run(sub_matches, project)?;
                } else {
                    c.run_no_project(sub_matches)?;
                }
            }
            None => println!("No command"),
        },
        None => println!("No command"),
    }

    crate::RENDERED.lock().unwrap().clean();

    if matches.is_present("time") {
        let elapsed = start.unwrap().elapsed();
        println!("Execution Took {}.{} Seconds", elapsed.as_secs(), elapsed.as_millis());
    }

    Ok(())
}
