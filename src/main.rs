use clap::{App};

use std::collections::HashMap;
use std::sync::Mutex;

#[macro_use]
pub mod macros;

mod build;
mod commands;
mod error;
mod flow;
mod project;
mod render;

pub use build::prebuild::RenderedFiles;
pub use error::{HEMTTError, FileErrorLineNumber, IOPathError};
pub use project::Project;
pub use build::{Addon, AddonLocation};
pub use flow::{Flow, Report, Task};

use crate::error::PrintableError;

lazy_static::lazy_static! {
    static ref RENDERED: Mutex<RenderedFiles> = Mutex::new(RenderedFiles::new());
    static ref CACHED: Mutex<RenderedFiles> = Mutex::new(RenderedFiles::new());
    static ref REPORTS: Mutex<HashMap<String, Report>> = Mutex::new(HashMap::new());
}

fn main() {
    let mut app = App::new("HEMTT")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS"))
                .about(env!("CARGO_PKG_DESCRIPTION"));

    let mut commands: Vec<Box<dyn crate::commands::Command>> = Vec::new();
    let mut hash_commands: HashMap<&str, &Box<dyn crate::commands::Command>> = HashMap::new();

    // Add commands here
    commands.push(Box::new(commands::Init {}));
    commands.push(Box::new(commands::Template {}));
    commands.push(Box::new(commands::Build {}));

    for command in commands.iter() {
        let (name, sub) = command.register();
        app = app.subcommand(sub);
        hash_commands.insert(name, command);
    }

    // rayon::ThreadPoolBuilder::new().num_threads(4).build_global().unwrap();

    let matches = app.get_matches();
    match matches.subcommand_name() {
        Some(v) => {
            match hash_commands.get(v) {
                Some(c) => {
                    if c.require_project() {
                        c.run(matches.subcommand_matches(v).unwrap(), Project::read().unwrap_or_print()).unwrap_or_print();
                    } else {
                        c.run_no_project(matches.subcommand_matches(v).unwrap()).unwrap_or_print();
                    }
                },
                None => println!("No command"),
            }
        },
        None => println!("No command"),
    }
    crate::RENDERED.lock().unwrap().clean();
}

use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn get_line_at(path: &Path, line_num: usize) -> Result<String, HEMTTError> {
    let file = File::open(path)?;
    let content = BufReader::new(&file);
    let mut lines = content.lines();
    Ok(lines.nth(line_num - 1).unwrap()?)
}
