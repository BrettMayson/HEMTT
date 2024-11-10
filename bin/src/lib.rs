use clap::CommandFactory;
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
mod progress;
pub mod report;
pub mod update;
pub mod utils;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
#[command(version = env!("HEMTT_VERSION"), about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[clap(flatten)]
    global: GlobalArgs,
}

#[derive(clap::Args)]
pub struct GlobalArgs {
    #[arg(global = true, long, short)]
    /// Number of threads, defaults to # of CPUs
    threads: Option<usize>,
    #[arg(global = true, short, action = clap::ArgAction::Count)]
    /// Verbosity level
    verbosity: u8,
    #[cfg(debug_assertions)]
    #[arg(global = true, long)]
    /// Directory to run in
    dir: Option<String>,
    #[cfg(debug_assertions)]
    #[arg(global = true, long, hide = true, action = clap::ArgAction::SetTrue)]
    /// we are in a test
    in_test: bool,
}

#[derive(clap::Subcommand)]
enum Commands {
    Book(commands::book::Command),
    New(commands::new::Command),
    Check(commands::check::Command),
    Dev(commands::dev::Command),
    Launch(commands::launch::Command),
    Build(commands::build::Command),
    Release(commands::release::Command),
    #[clap(alias = "ln")]
    Localization(commands::localization::Command),
    Script(commands::script::Command),
    Utils(commands::utils::Command),
    Value(commands::value::Command),
    Wiki(commands::wiki::Command),
}

/// Run the HEMTT CLI
///
/// # Errors
/// If the command fails
///
/// # Panics
/// If the number passed to `--threads` is not a valid number
pub fn execute(cli: &Cli) -> Result<(), Error> {
    // check for -v with no command and show version
    if cli.command.is_none() {
        if cli.global.verbosity > 0 {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("HEMTT_VERSION"));
            return Ok(());
        }
        Cli::command()
            .print_long_help()
            .expect("Failed to print help");
        std::process::exit(1);
    }

    #[cfg(debug_assertions)]
    let in_test = cli.global.in_test;
    #[cfg(not(debug_assertions))]
    let in_test = false;

    if !in_test && !matches!(cli.command, Some(Commands::Value(_))) {
        logging::init(
            cli.global.verbosity,
            !matches!(cli.command, Some(Commands::Utils(_))),
        );
    }

    #[cfg(debug_assertions)]
    if let Some(dir) = &cli.global.dir {
        std::env::set_current_dir(dir).expect("Failed to set current directory");
    }

    if !is_ci() {
        match update::check() {
            Ok(Some(version)) => {
                info!("HEMTT {version} is available, please update");
                if let Ok(path) = std::env::current_exe() {
                    trace!("HEMTT is installed at: {}", path.display());
                    if path.display().to_string().contains("\\Winget\\") {
                        info!(
                            "HEMTT is installed via winget, run `winget upgrade hemtt` to update"
                        );
                    }
                }
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

    if let Some(threads) = cli.global.threads {
        debug!("Using custom thread count: {threads}");
        if let Err(e) = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
        {
            error!("Failed to initialize thread pool: {e}");
        }
    }

    let report = match cli.command.as_ref().expect("Handled above") {
        Commands::Book(ref cmd) => commands::book::execute(cmd),
        Commands::New(ref cmd) => commands::new::execute(cmd, in_test),
        Commands::Check(ref cmd) => commands::check::execute(cmd),
        Commands::Dev(ref cmd) => commands::dev::execute(cmd, &[]),
        Commands::Launch(ref cmd) => commands::launch::execute(cmd),
        Commands::Build(ref cmd) => commands::build::execute(cmd),
        Commands::Release(ref cmd) => commands::release::execute(cmd),
        Commands::Localization(ref cmd) => commands::localization::execute(cmd),
        Commands::Script(ref cmd) => commands::script::execute(cmd),
        Commands::Utils(ref cmd) => commands::utils::execute(cmd),
        Commands::Value(ref cmd) => commands::value::execute(cmd),
        Commands::Wiki(ref cmd) => commands::wiki::execute(cmd),
    };

    match report {
        Ok(report) => {
            report.write_to_stdout();
            if !matches!(
                cli.command,
                Some(Commands::New(_) | Commands::Utils(_) | Commands::Wiki(_))
            ) {
                report.write_ci_annotations()?;
            }
            if report.failed() {
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Failed to execute command:\n{e}");
            std::process::exit(1);
        }
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

#[derive(clap::ValueEnum, Clone, Default, Debug, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TableFormat {
    /// ascii table
    #[default]
    Ascii,
    /// json
    Json,
    /// markdown table
    Markdown,
}
