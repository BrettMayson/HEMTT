use std::io::{Read, Write};

use hemtt_signing::BIPrivateKey;
use rand::{Rng as _, distr::Alphanumeric};

use crate::Error;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
/// Tools for working with private keys
pub struct Command {
    #[command(subcommand)]
    commands: Subcommands,
}

#[derive(clap::Subcommand)]
pub enum Subcommands {
    /// Generate a new HEMTT private key
    Generate {
        /// Authority name for the private key
        authority: String,
        /// Output file for the generated .hemttprivatekey
        output: Option<String>,
    },
    /// Convert an existing .biprivatekey to a .hemttprivatekey
    Convert {
        /// Input .biprivatekey file
        input: String,
        /// Output file for the converted .hemttprivatekey
        output: Option<String>,
    },
}

/// Execute the keys command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If the args are not present from clap
pub fn execute(cmd: &Command) -> Result<(), Error> {
    let (private_key, output) = match &cmd.commands {
        Subcommands::Generate { authority, output } => (
            BIPrivateKey::generate(1024, authority),
            output
                .clone()
                .unwrap_or_else(|| format!("{authority}.hemttprivatekey")),
        ),
        Subcommands::Convert { input, output } => {
            let key = BIPrivateKey::read(&mut fs_err::File::open(input)?);
            (
                key,
                output.clone().unwrap_or_else(|| {
                    let mut path = std::path::PathBuf::from(input);
                    path.set_extension("hemttprivatekey");
                    path.to_string_lossy().to_string()
                }),
            )
        }
    };
    if std::path::Path::new(&output).exists() {
        error!("Output file {output} already exists. Aborting to prevent overwrite.");
        std::process::exit(1);
    }
    let password = generate_password();
    let mut output = fs_err::File::create(output)?;
    let private_key = private_key?;
    private_key.write_encrypted(&mut output, &password)?;

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
        gitignore_file.write_all(format!("{gitignore_entry}\n").as_bytes())?;
        println!(".gitignore updated to exclude HEMTT private keys");
    }

    println!();
    println!("Add the following to your .hemtt/project.toml to use the key:");
    println!();
    println!("[signing]");
    println!("authority = \"{}\"", private_key.authority());
    println!(
        "private_key_hash = \"{}\"",
        private_key.validation_hash().expect("valid hash")
    );

    Ok(())
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
