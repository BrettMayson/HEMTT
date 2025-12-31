use std::io::{Read, Write};

use hemtt_signing::{BIPrivateKey, HEMTTPrivateKey, KDFParams};
use rand::{Rng as _, distr::Alphanumeric};

use crate::{Error, context::Context, modules::sign::get_git_first_hash, report::Report};

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Tools for working with private keys
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,
    /// KDF memory cost in MiB
    #[arg(long, default_value_t = default_mem_cost_mib())]
    mem_cost_mib: u32,
    /// KDF iterations
    #[arg(long, default_value_t = default_iterations())]
    iterations: u32,
    /// KDF parallelism
    #[arg(long, default_value_t = default_parallelism())]
    parallelism: u32,
}

#[derive(clap::Subcommand)]
pub enum Subcommands {
    /// Generate a new HEMTT private key
    Generate,
}

fn default_mem_cost_mib() -> u32 {
    KDFParams::default().mem_cost_kib / 1024
}

fn default_iterations() -> u32 {
    KDFParams::default().iterations
}

fn default_parallelism() -> u32 {
    KDFParams::default().parallelism
}

/// Execute the keys command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    match &cmd.commands {
        Subcommands::Generate => {
            let ctx = Context::new(None, crate::context::PreservePrevious::Keep, true)?;
            let authority = ctx.config().signing().authority().map_or_else(
                || ctx.config().prefix().to_string(),
                std::string::ToString::to_string,
            );

            let output = format!("{authority}.hemttprivatekey");
            if std::path::Path::new(&output).exists() {
                error!("Output file {output} already exists. Aborting to prevent overwrite.");
                std::process::exit(1);
            }

            warn!("Generating HEMTT private keys is for specific use cases only.");
            warn!("In nearly all cases, you should not use this command.");
            dialoguer::Confirm::new()
                .with_prompt("I fully understand the risks of using private keys")
                .default(true)
                .interact()?;

            let git_hash = get_git_first_hash()?;
            println!("Project:   {}", ctx.config().name());
            println!("Prefix:    {}", ctx.config().prefix());
            println!("Git:       {git_hash}");
            println!("Authority: {authority}");
            println!();
            warn!("The generated key will be usable ONLY with this project");
            if !dialoguer::Confirm::new()
                .with_prompt("Confirm")
                .default(false)
                .interact()?
            {
                return Ok(Report::new());
            }

            let hemtt_private_key = HEMTTPrivateKey {
                bi: BIPrivateKey::generate(1024, &authority)?,
                project: ctx.config().name().to_string(),
                prefix: ctx.config().prefix().to_string(),
                git_hash,
            };

            let kdf_params = KDFParams {
                mem_cost_kib: cmd.mem_cost_mib * 1024,
                iterations: cmd.iterations,
                parallelism: cmd.parallelism,
            };
            let password = generate_password();
            let mut output = fs_err::File::create(output)?;
            hemtt_private_key.write_encrypted(&mut output, &password, kdf_params)?;

            // Add to .gitignore if not already present
            let gitignore_path = std::path::Path::new(".gitignore");
            let mut gitignore_contents = String::new();
            if gitignore_path.exists() {
                fs_err::File::open(gitignore_path)?.read_to_string(&mut gitignore_contents)?;
            }
            let gitignore_entry = "*.hemttprivatekey";
            if !gitignore_contents
                .lines()
                .any(|line| line.trim() == gitignore_entry)
            {
                let mut gitignore_file = fs_err::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(gitignore_path)?;
                gitignore_file.write_all(format!("\n{gitignore_entry}\n").as_bytes())?;
                println!(".gitignore updated to exclude HEMTT private keys");
            }

            println!();
            println!("Add the following to your .hemtt/project.toml to use the key:");
            println!();
            println!("[signing]");
            println!(
                "private_key_hash = \"{}\"",
                hemtt_private_key.validation_hash().expect("valid hash")
            );
        }
    }

    Ok(Report::new())
}

fn generate_password() -> String {
    const LENGTH: usize = 32;
    let password: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(LENGTH)
        .collect::<Vec<u8>>()
        .chunks(LENGTH / 4)
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .expect("valid utf8")
        .join("-");
    println!("Store this password securely in a password manager");
    println!("Press ENTER once you have stored the password (it will be removed from your screen)");
    println!("Password: {password}");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("valid input");
    if cfg!(windows) {
        // clear the screen on windows
        std::process::Command::new("cmd")
            .args(["/C", "cls"])
            .status()
            .expect("valid clear screen");
    } else {
        // clear the screen on unix
        std::process::Command::new("clear")
            .status()
            .expect("valid clear screen");
    }
    println!("Paste the password to confirm");
    println!("(the password will not be shown on screen)");
    let mut tries = 0;
    loop {
        let entered = dialoguer::Password::new()
            .with_prompt("Confirm Password")
            .interact()
            .expect("valid input");
        if entered == password {
            break;
        }
        tries += 1;
        if tries >= 3 {
            error!("Too many incorrect attempts. Exiting.");
            std::process::exit(1);
        }
        println!("Passwords do not match. Please try again.");
    }
    password
}
