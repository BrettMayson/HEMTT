use std::{
    fs::create_dir,
    io::{IsTerminal, Write},
    path::Path,
};

use clap::{ArgMatches, Command};
use dialoguer::Input;

use crate::{
    commands::new::error::{
        bcne1_not_terminal::TerminalNotInput, bcne2_folder_exists::FolderExists,
    },
    error::Error,
    modules::Licenses,
    report::Report,
};

mod error;

#[must_use]
pub fn cli() -> Command {
    Command::new("new").about("Create a new project").arg(
        clap::Arg::new("name")
            .help("Name of the new mod")
            .required(true),
    )
}

/// Execute the new command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If a name is not provided, but this is usually handled by clap
pub fn execute(matches: &ArgMatches) -> Result<Report, Error> {
    let mut report = Report::new();

    let test_mode = !cfg!(not(debug_assertions)) && matches.get_flag("in-test");

    if !test_mode && !std::io::stdin().is_terminal() {
        report.error(TerminalNotInput::code());
        return Ok(report);
    }

    let name = matches
        .get_one::<String>("name")
        .expect("name to be set as required");
    let path = Path::new(&name);
    if path.exists() {
        report.error(FolderExists::code(name.to_string()));
        return Ok(report);
    }

    println!("Example: Advanced Banana Environment");
    let full_name: String = if test_mode {
        String::from("Advanced Banana Environment")
    } else {
        Input::new().with_prompt("Project Name").interact_text()?
    };

    println!("Example: ABE Team");
    let author: String = if test_mode {
        String::from("ABE Team")
    } else {
        Input::new().with_prompt("Author").interact_text()?
    };

    println!("Example: abe");
    let prefix: String = if test_mode {
        String::from("abe")
    } else {
        Input::new().with_prompt("Prefix").interact_text()?
    };

    let mainprefix: String = if test_mode {
        String::from("z")
    } else {
        Input::new()
            .with_prompt("Main Prefix")
            .with_initial_text("z")
            .interact_text()?
    };

    let license = if test_mode {
        Some(
            String::from_utf8(
                Licenses::get("apl-sa.txt")
                    .expect("apl-sa should exist")
                    .data
                    .to_vec(),
            )
            .expect("license should be utf8"),
        )
    } else {
        Licenses::select(&author)
    };

    create_dir(path)?;
    create_dir(path.join("addons"))?;

    git2::Repository::init(path)?;

    // Create .hemtt/project.toml
    let hemtt_path = path.join(".hemtt");
    create_dir(&hemtt_path)?;
    let mut file = std::fs::File::create(hemtt_path.join("project.toml"))?;
    file.write_all(
        format!("name = \"{full_name}\"\nauthor = \"{author}\"\nprefix = \"{prefix}\"\nmainprefix = \"{mainprefix}\"\n")
            .as_bytes(),
    )?;

    // Create .gitignore
    let mut file = std::fs::File::create(path.join(".gitignore"))?;
    file.write_all(b"*.pbo\n.hemttout\nhemtt\nhemtt.exe\n*.biprivatekey\n")?;

    // Create LICENSE
    if let Some(license) = license {
        let mut file = std::fs::File::create(path.join("LICENSE"))?;
        file.write_all(license.as_bytes())?;
    }

    Ok(report)
}
