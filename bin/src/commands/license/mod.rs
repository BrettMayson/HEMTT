use std::path::Path;

use dialoguer::{Confirm, Input};

use crate::{context::Context, error::Error, modules::Licenses, report::Report};

#[derive(clap::Parser)]
#[command(verbatim_doc_comment)]
/// Add or update a license file in the current project
///
/// `hemtt license` is used to add or update a LICENSE file in your project.
///
/// You can either provide a license name as an argument, or run the command
/// interactively to select from a list of available licenses.
///
/// Available license names:
/// - apl-sa (Arma Public License Share Alike)
/// - apl (Arma Public License)
/// - apl-nd (Arma Public License No Derivatives)
/// - apache (Apache 2.0)
/// - gpl (GNU GPL v3)
/// - mit (MIT)
/// - unlicense (Unlicense)
pub struct Command {
    #[clap(name = "name", verbatim_doc_comment)]
    /// The name of the license to add
    ///
    /// If not provided, you will be prompted to select a license interactively.
    ///
    /// Examples: `apl-sa`, `mit`, `apache`
    name: Option<String>,
}

/// Execute the license command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If there is a problem with dialoguer
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    let report = Report::new();

    // If a license name is provided, validate it early
    if let Some(name) = &cmd.name {
        // Quick validation before prompting for anything
        if Licenses::get_by_name(name, "dummy").is_none() {
            eprintln!("Error: Unknown license name: {name}");
            eprintln!("\nAvailable licenses:");
            eprintln!("  - apl-sa (Arma Public License Share Alike)");
            eprintln!("  - apl (Arma Public License)");
            eprintln!("  - apl-nd (Arma Public License No Derivatives)");
            eprintln!("  - apache (Apache 2.0)");
            eprintln!("  - gpl (GNU GPL v3)");
            eprintln!("  - mit (MIT)");
            eprintln!("  - unlicense (Unlicense)");
            return Ok(report);
        }
    }

    // Get author from project.toml if it exists, otherwise prompt
    let author = get_author_from_project_or_prompt()?;

    // Check if LICENSE file already exists and prompt for confirmation
    let license_path = Path::new("LICENSE");
    if license_path.exists() {
        let confirm = Confirm::new()
            .with_prompt("LICENSE file already exists. Do you want to overwrite it?")
            .default(false)
            .interact()?;

        if !confirm {
            println!("License update cancelled.");
            return Ok(report);
        }
    }

    let license_text = if let Some(name) = &cmd.name {
        // Use the provided license name (already validated above)
        Licenses::get_by_name(name, &author).expect("License name already validated")
    } else {
        // Interactive selection
        if let Some(text) = Licenses::select(&author) {
            text
        } else {
            println!("No license selected.");
            return Ok(report);
        }
    };

    // Write the license file
    write_license_file(&license_text, license_path)?;

    println!("License file created successfully at: LICENSE");

    Ok(report)
}

fn get_author_from_project_or_prompt() -> Result<String, Error> {
    // Try to read the project config properly
    match Context::read_project() {
        Ok(config) => {
            if let Some(author) = config.author() {
                return Ok(author.clone());
            }
        }
        Err(Error::ConfigNotFound) => {
            // No project config, will prompt for author
        }
        Err(e) => {
            // Other error, log but continue to prompt
            warn!("Failed to read project config: {}", e);
        }
    }

    // Prompt for author if not found in project.toml
    prompt_for_author()
}

fn prompt_for_author() -> Result<String, Error> {
    println!("Example: John Doe");
    Ok(Input::new().with_prompt("Author").interact_text()?)
}

/// Write a license file to the given path
///
/// # Errors
/// Returns an error if the file cannot be created or written to
pub fn write_license_file(
    license_text: &str,
    path: &std::path::Path,
) -> Result<(), std::io::Error> {
    use std::io::Write;
    let mut file = std::fs::File::create(path)?;
    file.write_all(license_text.as_bytes())?;
    Ok(())
}
