use clap::{App};

#[cfg(windows)]
use ansi_term;

use std::collections::HashMap;

#[macro_use]
pub mod macros;

use hemtt::*;

use crate::error::PrintableError;

fn main() {
    if cfg!(windows) {
        ansi_support();
    }

    println!("Version {}", env!("CARGO_PKG_VERSION"));

    let mut app = App::new("HEMTT")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS"))
                .about(env!("CARGO_PKG_DESCRIPTION"))
                .arg(clap::Arg::with_name("jobs")
                    .global(true)
                    .help("Number of parallel jobs to perform")
                    .takes_value(true)
                    .long("jobs")
                    .short("j"))
                .arg(clap::Arg::with_name("debug")
                    .global(true)
                    .help("Turn debugging information on")
                    .long("debug")
                    .short("d"));

    let mut commands: Vec<Box<dyn Command>> = Vec::new();
    let mut hash_commands: HashMap<&str, &Box<dyn Command>> = HashMap::new();

    // Add commands here
    commands.push(Box::new(commands::Init {}));
    commands.push(Box::new(commands::Template {}));
    commands.push(Box::new(commands::Build {}));
    commands.push(Box::new(commands::Pack {}));
    commands.push(Box::new(commands::Status {}));

    for command in commands.iter() {
        let (name, sub) = command.register();
        app = app.subcommand(sub);
        hash_commands.insert(name, command);
    }

    let matches = app.get_matches();

    rayon::ThreadPoolBuilder::new().num_threads(
        if let Some(jobs) = matches.value_of("jobs") { usize::from_str_radix(jobs, 10).unwrap_or_print() } else { num_cpus::get() }
    ).build_global().unwrap();

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
