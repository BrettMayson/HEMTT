use std::{
    io::{IsTerminal, Write},
    path::Path,
};

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

#[derive(clap::Parser)]
#[command(arg_required_else_help = true, verbatim_doc_comment)]
/// Create a new project
///
/// `hemtt new` is used to create a new mod. It will create a new folder with the name you provide, and create some starting files.
///
/// ## Interactive Setup
///
/// The command will interactively prompt for:
///
/// - The full name of your mod  
/// - The author of your mod  
/// - The prefix of your mod (used for addon naming)
/// - The main prefix of your mod (folder prefix like 'z')
/// - A license for your mod
///
/// ## Generated Structure
///
/// Creates a complete project structure with:
/// - `.hemtt/project.toml` - Project configuration
/// - `addons/main/` - Example addon structure
/// - `LICENSE` - Selected license file
/// - `.gitignore` - Standard git ignore patterns
/// - README template  
pub struct Command {
    #[clap(name = "name", verbatim_doc_comment)]
    /// The name of the new project
    ///
    /// This will create a new folder with the name you provide in the current directory.
    /// It should be a valid folder name, using only letters, numbers, and underscores.
    /// This name is typically lowercase and used for the directory, not the display name.
    ///
    /// Example: `hemtt new my_mod`
    name: String,
}

/// Execute the new command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If a name is not provided, but this is usually handled by clap
pub fn execute(cmd: &Command, in_test: bool) -> Result<Report, Error> {
    let mut report = Report::new();

    let test_mode = !cfg!(not(debug_assertions)) && in_test;

    if !test_mode && !std::io::stdin().is_terminal() {
        report.push(TerminalNotInput::code());
        return Ok(report);
    }

    let path = Path::new(&cmd.name);
    if path.exists() {
        report.push(FolderExists::code(cmd.name.clone()));
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
        Some(include_str!("../../modules/new/licenses/mit.txt").to_string())
    } else {
        Licenses::select(&author)
    };

    fs_err::create_dir(path)?;
    fs_err::create_dir(path.join("addons"))?;

    git2::Repository::init(path)?;

    // Create .hemtt/project.toml
    let hemtt_path = path.join(".hemtt");
    fs_err::create_dir(&hemtt_path)?;
    let mut file = fs_err::File::create(hemtt_path.join("project.toml"))?;
    file.write_all(
        format!("name = \"{full_name}\"\nauthor = \"{author}\"\nprefix = \"{prefix}\"\nmainprefix = \"{mainprefix}\"\n")
            .as_bytes(),
    )?;

    // Create .gitignore
    let mut file = fs_err::File::create(path.join(".gitignore"))?;
    file.write_all(b"*.pbo\n.hemttout\nhemtt\nhemtt.exe\n*.biprivatekey\n")?;

    // Create LICENSE
    if let Some(license) = license {
        crate::commands::license::write_license_file(&license, &path.join("LICENSE"))?;
    }

    Ok(report)
}
