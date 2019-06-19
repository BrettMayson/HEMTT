use clap::{App};

use std::collections::HashMap;

#[macro_use]
pub mod macros;

mod commands;
mod project;

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
                        c.run(matches.subcommand_matches(v).unwrap(), project::Project::read().unwrap());
                    } else {
                        c.run_no_project(matches.subcommand_matches(v).unwrap());
                    }
                },
                None => println!("No command"),
            }
        },
        None => println!("No command"),
    }
}
