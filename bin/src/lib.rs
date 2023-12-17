use clap::{ArgAction, ArgMatches, Command};
pub use error::Error;

#[macro_use]
extern crate tracing;

pub mod commands;
pub mod context;
pub mod error;
pub mod executor;
pub mod link;
pub mod logging;
pub mod modules;
pub mod report;
pub mod update;
pub mod utils;

#[must_use]
pub fn cli() -> Command {
    #[allow(unused_mut)]
    let mut global = Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("HEMTT_VERSION"))
        .subcommand_required(false)
        .arg_required_else_help(true)
        .subcommand(commands::new::cli())
        .subcommand(commands::dev::cli())
        .subcommand(commands::build::cli())
        .subcommand(commands::launch::cli())
        .subcommand(commands::release::cli())
        .subcommand(commands::script::cli())
        .subcommand(commands::utils::cli())
        .arg(
            clap::Arg::new("threads")
                .global(true)
                .help("Number of threads, defaults to # of CPUs")
                .action(ArgAction::Set)
                .long("threads")
                .short('t'),
        )
        .arg(
            clap::Arg::new("verbosity")
                .global(true)
                .help("Verbosity level")
                .action(ArgAction::Count)
                .short('v'),
        );
    #[cfg(debug_assertions)]
    {
        global = global
            .arg(
                clap::Arg::new("in-test")
                    .global(true)
                    .help("we are in a test")
                    .action(ArgAction::SetTrue)
                    .long("in-test"),
            )
            .arg(
                clap::Arg::new("dir")
                    .global(true)
                    .help("directory to run in")
                    .action(ArgAction::Set)
                    .long("dir"),
            );
    }
    global
}

/// Run the HEMTT CLI
///
/// # Errors
/// If the command fails
///
/// # Panics
/// If the number passed to `--threads` is not a valid number
pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    if cfg!(not(debug_assertions)) || !matches.get_flag("in-test") {
        logging::init(
            matches.get_count("verbosity"),
            matches.subcommand_name() != Some("utils"),
        );
    }
    if let Some(dir) = matches.get_one::<String>("dir") {
        std::env::set_current_dir(dir).expect("Failed to set current directory");
    }

    if !is_ci() {
        match update::check() {
            Ok(Some(version)) => {
                info!("HEMTT {version} is available, please update");
            }
            Err(e) => {
                error!("Failed to check for updates: {e}");
            }
            _ => {}
        }
    }

    trace!("version: {}", env!("HEMTT_VERSION"));
    trace!("platform: {}", std::env::consts::OS);

    trace!("args: {:#?}", std::env::args().collect::<Vec<String>>());

    if let Some(threads) = matches.get_one::<String>("threads") {
        if let Err(e) = rayon::ThreadPoolBuilder::new()
            .num_threads(threads.parse::<usize>().unwrap())
            .build_global()
        {
            error!("Failed to initialize thread pool: {e}");
        }
    }
    let report = match matches.subcommand() {
        Some(("new", matches)) => commands::new::execute(matches).map(Some),
        Some(("dev", matches)) => commands::dev::execute(matches, &[]).map(Some),
        Some(("build", matches)) => commands::build::execute(matches)
            .map_err(std::convert::Into::into)
            .map(Some),
        Some(("release", matches)) => commands::release::execute(matches)
            .map_err(std::convert::Into::into)
            .map(Some),
        Some(("launch", matches)) => commands::launch::execute(matches)
            .map_err(std::convert::Into::into)
            .map(Some),
        Some(("script", matches)) => commands::script::execute(matches)
            .map_err(std::convert::Into::into)
            .map(Some),
        Some(("utils", matches)) => commands::utils::execute(matches)
            .map_err(std::convert::Into::into)
            .map(Some),
        _ => unreachable!(),
    };
    if let Some(report) = report? {
        report.write_to_stdout();
        report.write_ci_annotations()?;
    }
    Ok(())
}

#[must_use]
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
