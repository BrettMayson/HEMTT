use std::{fs::create_dir, io::Write, path::Path};

use atty::Stream;
use clap::{ArgMatches, Command};
use dialoguer::Input;

use crate::{error::Error, modules::Licenses};

#[must_use]
pub fn cli() -> Command {
    Command::new("new").about("Create a new project").arg(
        clap::Arg::new("name")
            .help("Name of the new mod")
            .required(true),
    )
}

pub fn execute(matches: &ArgMatches) -> Result<(), Error> {
    if !atty::is(Stream::Stdin) {
        return Err(Error::NewNoInput);
    }

    let name = matches
        .get_one::<String>("name")
        .expect("name to be set as required");
    let path = Path::new(&name);
    if path.exists() {
        return Err(Error::NewFolderExists(name.to_string()));
    }

    println!("Example: Advanced Banana Environment");
    let full_name: String = Input::new().with_prompt("Project Name").interact_text()?;

    println!("Example: ABE Team");
    let author: String = Input::new().with_prompt("Author").interact_text()?;

    println!("Example: abe");
    let prefix: String = Input::new().with_prompt("Prefix").interact_text()?;

    let mainprefix: String = Input::new()
        .with_prompt("Main Prefix")
        .with_initial_text("z")
        .interact_text()?;

    let license = Licenses::select(&author);

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

    Ok(())
}
