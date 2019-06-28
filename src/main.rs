use clap::{App};

#[cfg(windows)]
use ansi_term;

use std::collections::HashMap;
use std::sync::{Mutex, Arc};

#[macro_use]
pub mod macros;

mod build;
mod commands;
mod error;
mod files;
mod flow;
mod project;
mod render;

pub use build::{Addon, AddonLocation};
pub use error::{HEMTTError, FileErrorLineNumber, IOPathError};
pub use files::{FileCache, RenderedFiles};
pub use flow::{Flow, Report, Task, Step};
pub use project::Project;

use crate::error::PrintableError;

lazy_static::lazy_static! {
    static ref RENDERED: Arc<Mutex<RenderedFiles>> = Arc::new(Mutex::new(RenderedFiles::new()));
    static ref CACHED: Arc<Mutex<FileCache>> = Arc::new(Mutex::new(FileCache::new()));
    static ref REPORTS: Arc<Mutex<HashMap<String, Report>>> = Arc::new(Mutex::new(HashMap::new()));
}

fn main() {
    if cfg!(windows) {
        ansi_support();
    }

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

    rayon::ThreadPoolBuilder::new().num_threads(12).build_global().unwrap();

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

#[cfg(windows)]
fn ansi_support() {
    // Attempt to enable ANSI support in terminal
    // Disable colored output if failed
    if ansi_term::enable_ansi_support().is_err() {
        colored::control::set_override(false);
    }
}

#[cfg(not(windows))]
fn ansi_support() {
    unreachable!();
}
