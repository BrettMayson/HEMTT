use clap::{App};

use std::collections::HashMap;

#[macro_use]
pub mod macros;

mod build;
mod checks;
mod commands;
mod error;
mod flow;
mod project;

use crate::error::PrintableError;

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

    let matches = app.get_matches();
    match matches.subcommand_name() {
        Some(v) => {
            match hash_commands.get(v) {
                Some(c) => {
                    if c.require_project() {
                        c.run(matches.subcommand_matches(v).unwrap(), project::Project::read().unwrap()).unwrap_or_print();
                    } else {
                        c.run_no_project(matches.subcommand_matches(v).unwrap()).unwrap_or_print();
                    }
                },
                None => println!("No command"),
            }
        },
        None => println!("No command"),
    }
}
