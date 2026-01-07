use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

use clap::CommandFactory;
use clap_complete::{Shell, generate_to};

use crate::{Cli, Error, report::Report};

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Manage HEMTT features installed on your system
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,

    #[clap(flatten)]
    global: crate::GlobalArgs,
}

#[derive(clap::Subcommand)]
enum Subcommands {
    /// Install shell completions
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    Powershell,
}

/// Execute the manage command
///
/// # Errors
/// Will not return an error
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    match &cmd.commands {
        Subcommands::Completions { shell } => {
            clap_complete::generate(
                *shell,
                &mut Cli::command(),
                env!("CARGO_PKG_NAME"),
                &mut std::io::stdout(),
            );
        }
        Subcommands::Powershell => {
            install_powershell_completions_sourced()?;
        }
    }
    std::process::exit(0);
}

#[allow(dead_code)]
fn install_powershell_completions_sourced() -> std::io::Result<()> {
    if !cfg!(windows) {
        eprintln!("PowerShell completions can only be installed on Windows");
        std::process::exit(1);
    }

    let script_path = dirs::config_dir()
        .expect("Could not locate config directory (APPDATA)")
        .join("hemtt")
        .join("_hemtt.ps1");

    let script_dir = script_path
        .parent()
        .expect("Could not determine script directory");
    fs_err::create_dir_all(script_dir)?;

    generate_to(
        Shell::PowerShell,
        &mut Cli::command(),
        env!("CARGO_PKG_NAME"),
        script_dir,
    )?;

    let profile_path =
        get_powershell_profile_path().expect("Could not determine PowerShell profile path");

    if let Some(parent) = profile_path.parent() {
        fs_err::create_dir_all(parent)?;
    }
    if !profile_path.exists() {
        File::create(&profile_path)?;
    }

    let import_line = r#". "$env:APPDATA\hemtt\_hemtt.ps1""#;

    let mut profile_content = String::new();
    File::open(&profile_path)?.read_to_string(&mut profile_content)?;

    if profile_content.contains(import_line) {
        info!("PowerShell completions already configured in your profile.");
        return Ok(());
    }
    let mut file = OpenOptions::new().append(true).open(&profile_path)?;
    writeln!(
        file,
        "\n# hemtt manage completions powershell\n{import_line}"
    )?;

    debug!("Completion script saved to: {}", script_path.display());
    info!("Restart PowerShell or run:  . $PROFILE");

    Ok(())
}

fn get_powershell_profile_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| {
        home.join("Documents")
            .join("WindowsPowerShell")
            .join("Microsoft.PowerShell_profile.ps1")
    })
}
