use std::collections::HashMap;
use std::time::Instant;

use clap::App;

#[cfg(windows)]
use ansi_term;

#[macro_use]
pub mod macros;

mod startup;

use hemtt::*;

use crate::error::PrintableError;

fn main() {
    if cfg!(windows) {
        ansi_support();
    }

    let mut version = env!("CARGO_PKG_VERSION").to_string();
    if let Some(v) = option_env!("GIT_HASH") {
        version.push_str("-");
        version.push_str(v);
    }
    if cfg!(debug_assertions) {
        version.push_str("-debug");
    }

    let mut app = App::new("HEMTT")
        .version(version.as_ref())
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            clap::Arg::with_name("jobs")
                .global(true)
                .help("Number of parallel jobs to perform")
                .takes_value(true)
                .long("jobs")
                .short("j"),
        )
        .arg(
            clap::Arg::with_name("debug")
                .global(true)
                .help("Turn debugging information on")
                .long("debug")
                .short("d"),
        )
        .arg(
            clap::Arg::with_name("time")
                .global(true)
                .help("Time the execution")
                .long("time"),
        )
        .arg(
            clap::Arg::with_name("ci") // This is not actually checked by clap, see lib.rs
                .global(true)
                .help("Run in CI mode")
                .long("ci"),
        );

    let mut commands: Vec<Box<dyn Command>> = Vec::new();
    let mut hash_commands: HashMap<String, &Box<dyn Command>> = HashMap::new();

    // Add commands here
    commands.push(Box::new(commands::Init {}));
    commands.push(Box::new(commands::Template {}));
    commands.push(Box::new(commands::Build {}));
    commands.push(Box::new(commands::Pack {}));
    commands.push(Box::new(commands::Clean {}));
    commands.push(Box::new(commands::Status {}));

    // Add utilities here
    commands.push(Box::new(utilities::Translation {}));
    commands.push(Box::new(utilities::MissionGenerate {}));

    for command in commands.iter() {
        let sub = command.register();
        hash_commands.insert(sub.get_name().to_owned(), command);
        app = app.subcommand(sub);
    }

    let matches = app.get_matches();

    let start = if matches.is_present("time") {
        Some(Instant::now())
    } else {
        None
    };

    rayon::ThreadPoolBuilder::new()
        .num_threads(if let Some(jobs) = matches.value_of("jobs") {
            usize::from_str_radix(jobs, 10).unwrap_or_print()
        } else {
            num_cpus::get()
        })
        .build_global()
        .unwrap();

    match matches.subcommand_name() {
        Some(v) => match hash_commands.get(v) {
            Some(c) => {
                println!("HEMTT {}", version);
                println!("Environment: {}", project::environment());
                println!();
                startup::startup();
                let sub_matches = matches.subcommand_matches(v).unwrap();
                if c.require_project() {
                    c.run(sub_matches, Project::read().unwrap_or_print()).unwrap_or_print();
                } else {
                    c.run_no_project(sub_matches).unwrap_or_print();
                }
            }
            None => println!("No command"),
        },
        None => println!("No command"),
    }

    crate::RENDERED.lock().unwrap().clean();

    if matches.is_present("time") {
        let elapsed = start.unwrap().elapsed();
        println!("Execution Took {}.{} Seconds", elapsed.as_secs(), elapsed.as_millis());
    }
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
