use std::path::Path;

use dialoguer::Input;

use crate::{
    error::Error,
    modules::Licenses,
    report::Report,
};

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
pub fn execute(cmd: &Command) -> Result<Report, Error> {
    let report = Report::new();

    // Get author from project.toml if it exists, otherwise prompt
    let author = get_author_from_project_or_prompt()?;

    let license_text = if let Some(name) = &cmd.name {
        // Use the provided license name
        match Licenses::get_by_name(name, &author) {
            Some(text) => text,
            None => {
                eprintln!("Error: Unknown license name: {}", name);
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
    } else {
        // Interactive selection
        match Licenses::select(&author) {
            Some(text) => text,
            None => {
                println!("No license selected.");
                return Ok(report);
            }
        }
    };

    // Write the license file
    let license_path = Path::new("LICENSE");
    Licenses::write_license_file(&license_text, license_path)?;

    println!("License file created successfully at: LICENSE");

    Ok(report)
}

fn get_author_from_project_or_prompt() -> Result<String, Error> {
    let project_path = std::env::current_dir()?.join(".hemtt").join("project.toml");
    
    if project_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&project_path) {
            // Simple TOML parsing to extract author field
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("author") {
                    if let Some(value) = line.split('=').nth(1) {
                        let author = value.trim().trim_matches('"').trim_matches('\'');
                        if !author.is_empty() {
                            return Ok(author.to_string());
                        }
                    }
                }
            }
        }
    }
    
    // Prompt for author if not found in project.toml
    prompt_for_author()
}

fn prompt_for_author() -> Result<String, Error> {
    println!("Example: John Doe");
    Ok(Input::new().with_prompt("Author").interact_text()?)
}
