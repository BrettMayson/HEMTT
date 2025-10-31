use clap::CommandFactory;
pub use error::Error;

#[macro_use]
extern crate tracing;

pub mod commands;
pub mod context;
pub mod controller;
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

#[derive(Debug, Clone, clap::Args)]
pub struct GlobalArgs {
    #[arg(global = true, long, short)]
    /// Number of threads, defaults to # of CPUs
    threads: Option<usize>,
    #[arg(global = true, short, action = clap::ArgAction::Count)]
    /// Verbosity level
    verbosity: u8,
    #[arg(global = true, hide = true, long)]
    /// Directory to run in
    dir: Option<String>,
    #[cfg(debug_assertions)]
    #[arg(global = true, long, hide = true, action = clap::ArgAction::SetTrue)]
    /// we are in a test
    in_test: bool,
    #[arg(global = true, long, hide = true, action = clap::ArgAction::SetTrue)]
    exp_bin_cache: bool,
}

#[derive(clap::Subcommand)]
enum Commands {
    Book(commands::book::Command),
    New(commands::new::Command),
    Check(commands::check::Command),
    Dev(commands::dev::Command),
    Launch(commands::launch::Command),
    License(commands::license::Command),
    Build(commands::build::Command),
    Release(commands::release::Command),
    #[clap(alias = "ln")]
    Localization(commands::localization::Command),
    Script(commands::script::Command),
    Utils(commands::utils::Command),
    Value(commands::value::Command),
    Wiki(commands::wiki::Command),
    Manage(commands::manage::Command),
    #[cfg(windows)]
    Photoshoot(commands::photoshoot::Command),
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

    if let Some(dir) = &cli.global.dir {
        std::env::set_current_dir(dir).expect("Failed to set current directory");
    }

    #[cfg(debug_assertions)]
    let in_test = cli.global.in_test;
    #[cfg(not(debug_assertions))]
    let in_test = false;

    if !in_test && !matches!(cli.command, Some(Commands::Value(_))) {
        logging::init(
            cli.global.verbosity,
            !matches!(
                cli.command,
                Some(
                    Commands::Utils(_)
                        | Commands::Wiki(_)
                        | Commands::New(_)
                        | Commands::Book(_)
                        | Commands::License(_)
                        | Commands::Manage(_)
                )
            ),
        )?;
    }

    let update_thread = std::thread::spawn(check_for_update);

    trace!("version: {}", env!("HEMTT_VERSION"));
    trace!("platform: {}", std::env::consts::OS);

    trace!("args: {:#?}", std::env::args().collect::<Vec<String>>());

    if let Some(threads) = cli.global.threads {
        debug!("Using custom thread count: {threads}");
        let mut builder = rayon::ThreadPoolBuilder::new().num_threads(threads);
        if threads == 1 {
            // helps with profiling to just use the main thread
            builder = builder.use_current_thread();
        }
        if let Err(e) = builder.build_global() {
            error!("Failed to initialize thread pool: {e}");
        }
    }

    let report = match cli.command.as_ref().expect("Handled above") {
        Commands::Book(cmd) => commands::book::execute(cmd),
        Commands::New(cmd) => commands::new::execute(cmd, in_test),
        Commands::Check(cmd) => commands::check::execute(cmd),
        Commands::Dev(cmd) => commands::dev::execute(cmd, &[], false).map(|(r, _)| r),
        Commands::Launch(cmd) => commands::launch::execute(cmd),
        Commands::License(cmd) => commands::license::execute(cmd),
        Commands::Build(cmd) => commands::build::execute(cmd),
        Commands::Release(cmd) => commands::release::execute(cmd),
        Commands::Localization(cmd) => commands::localization::execute(cmd),
        Commands::Script(cmd) => commands::script::execute(cmd),
        Commands::Utils(cmd) => commands::utils::execute(cmd),
        Commands::Value(cmd) => commands::value::execute(cmd),
        Commands::Wiki(cmd) => commands::wiki::execute(cmd),
        Commands::Manage(cmd) => commands::manage::execute(cmd),
        #[cfg(windows)]
        Commands::Photoshoot(cmd) => commands::photoshoot::execute(cmd),
    };

    match report {
        Ok(report) => {
            report.write_to_stdout();
            if !matches!(
                cli.command,
                Some(Commands::New(_) | Commands::License(_) | Commands::Utils(_) | Commands::Wiki(_))
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

    match update_thread.join() {
        Err(e) => {
            error!("Failed to check for updates: {e:?}");
        }
        Ok(Some(lines)) => {
            for line in lines {
                tracing::info!("{}", line);
            }
        }
        Ok(None) => {}
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

/// Check for updates to HEMTT
///
/// # Returns
/// If an update is available, a message to display to the user
///
/// # Panics
/// If the user's home directory does not exist
fn check_for_update() -> Option<Vec<String>> {
    if is_ci() {
        return None;
    }
    let mut out = Vec::new();
    match update::check() {
        Ok(Some(version)) => {
            out.push(format!("HEMTT {version} is available, please update"));
        }
        Err(e) => {
            error!("Failed to check for updates: {e}");
            return None;
        }
        _ => return None,
    }
    let Ok(path) = std::env::current_exe() else {
        return Some(out);
    };
    trace!("HEMTT is installed at: {}", path.display());
    let os = std::env::consts::OS;
    let (message, filter) = match os {
        "windows" => (
            "HEMTT is installed via winget, run `winget upgrade hemtt` to update",
            "\\Winget\\".to_string(),
        ),
        "linux" | "macos" => (
            "HEMTT is installed in home directory, run `curl -sSf https://hemtt.dev/install.sh | sh` to update",
            {
                let mut home = dirs::home_dir().expect("home directory exists");
                if os == "linux" {
                    home = home.join(".local");
                }
                home.join("bin").display().to_string()
            },
        ),
        _ => return Some(out),
    };

    if path.display().to_string().contains(&filter) {
        out.push(message.to_string());
    }
    Some(out)
}

#[derive(clap::ValueEnum, Clone, Default, Debug, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TableFormat {
    /// an ascii table for the terminal
    #[default]
    Ascii,
    /// compact json, ideal for machines
    Json,
    /// pretty json, ideal for humans
    PrettyJson,
    /// a markdown table, ideal for documentation or GitHub
    Markdown,
}
