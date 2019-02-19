use serde::Deserialize;
use docopt::Docopt;
use colored::*;

#[cfg(windows)]
use ansi_term;

use self_update;

use std::collections::{HashSet};
use std::io::{stdin, stdout, Write, Error};
use std::path::Path;

mod build;
mod error;
mod files;
mod project;

use crate::error::*;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const HEMTT_FILE: &str = "hemtt.json";

const USAGE: &'static str = "
HEMTT, a simple to use build manager for Arma 3 mods using the CBA project structure

Usage:
    hemtt init
    hemtt create
    hemtt addon <name>
    hemtt build [--release] [--force] [--nowarn]
    hemtt clean [--force]
    hemtt update
    hemtt (-h | --help)
    hemtt --version

Commands:
    init        Initialize a project file in the current directory
    create      Create a new project using the CBA project structure
    addon       Create a new addon folder
    build       Build the project
    clean       Clean build files
    update      Update HEMTT

Options:
    -v --verbose        Enable verbose output
    -f --force          Overwrite target files
       --nowarn         Suppress armake2 warnings
    -h --help           Show usage information and exit
       --version        Show version number and exit
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_init: bool,
    cmd_create: bool,
    cmd_addon: bool,
    cmd_build: bool,
    cmd_clean: bool,
    cmd_update: bool,
    flag_verbose: bool,
    flag_force: bool,
    flag_nowarn: bool,
    flag_version: bool,
    flag_release: bool,
    arg_name: String,
}

fn input(text: &str) -> String {
    let mut s = String::new();
    print!("{}: ",text);
    stdout().flush().unwrap();
    stdin().read_line(&mut s).expect("Did not enter a valid string");
    if let Some('\n')=s.chars().next_back() {
        s.pop();
    }
    if let Some('\r')=s.chars().next_back() {
        s.pop();
    }
    s
}

fn run_command(args: &Args) -> Result<(), Error> {
    if args.cmd_init {
        check(true, args.flag_force).print_error(true);
        init().unwrap();
        Ok(())
    } else if args.cmd_create {
        check(true, args.flag_force).print_error(true);
        let p = init().unwrap();
        let main = "main".to_owned();
        files::modcpp(&p).unwrap();
        files::create_addon(&main, &p).unwrap();
        files::scriptmodhpp(&p).unwrap();
        files::scriptversionhpp(&p).unwrap();
        files::scriptmacroshpp(&p).unwrap();
        files::script_component(&main, &p).unwrap();
        files::pboprefix(&main, &p).unwrap();
        files::configcpp(&main, &p).unwrap();
        files::create_include().unwrap();
        Ok(())
    } else if args.cmd_addon {
        check(false, args.flag_force).print_error(true);
        let p = project::get_project().unwrap();
        if Path::new(&format!("addons/{}", args.arg_name)).exists() {
            return Err(error!("{} already exists", args.arg_name.bold()));
        }
        println!("Creating addon: {}", args.arg_name);
        files::create_addon(&args.arg_name, &p).unwrap();
        files::pboprefix(&args.arg_name, &p).unwrap();
        files::script_component(&args.arg_name, &p).unwrap();
        files::configcpp(&args.arg_name, &p).unwrap();
        files::xeh(&args.arg_name, &p).unwrap();
        Ok(())
    } else if args.cmd_build {
        check(false, args.flag_force).print_error(true);
        let p = project::get_project().unwrap();
        if args.flag_force {
            files::clear_pbos(&p).unwrap();
        }
        if !args.flag_nowarn {
            unsafe {
                armake2::error::WARNINGS_MUTED = Some(HashSet::new());
            }
        }
        if args.flag_release {
            let version = match &p.version {
                Some(v) => v,
                None => panic!("Unable to determine version number"),
            };
            if args.flag_force {
                files::clear_release(&version).unwrap();
            }
            build::release(&p, &version).print_error(true);
            println!("  {} {} v{}", "Finished".green().bold(), &p.name, version);
        } else {
            build::build(&p).unwrap();
            println!("  {} {}", "Finished".green().bold(), &p.name);
        }
        if !args.flag_nowarn {
            armake2::error::print_warning_summary();
        }
        Ok(())
    } else if args.cmd_clean {
        check(false, args.flag_force).print_error(true);
        let p = project::get_project().unwrap();
        files::clear_pbos(&p).unwrap();
        if args.flag_force {
            files::clear_releases().unwrap();
        }
        Ok(())
    } else if args.cmd_update {
        let target = self_update::get_target().unwrap();
        let status = self_update::backends::github::Update::configure().unwrap()
            .repo_owner("SynixeBrett")
            .repo_name("HEMTT")
            .target(&target)
            .bin_name("hemtt")
            .show_download_progress(true)
            .current_version(VERSION)
            .build().unwrap()
            .update().unwrap();
        println!("Using Version: {}", status.version());
        Ok(())
    } else {
        unreachable!()
    }
}

fn main() {
    if cfg!(windows) {
        ansi_support();
    }

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("HEMTT Version {}", VERSION);
        std::process::exit(0);
    }

    run_command(&args).print_error(true);
}

fn check(write: bool, force: bool) -> Result<(), std::io::Error> {
    if Path::new(HEMTT_FILE).exists() && write && !force {
        Err(error!("HEMTT Project already exists in the current directory"))
    } else if Path::new(HEMTT_FILE).exists() && write && force {
        Ok(())
    } else if !Path::new(HEMTT_FILE).exists() && !write {
        Err(error!("A HEMTT Project does not exist in the current directory"))
    } else {
        Ok(())
    }
}

fn init() -> Result<crate::project::Project, std::io::Error> {
    let name = input("Project Name (My Cool Mod)");
    let prefix = input("Prefix (MCM)");
    let author = input("Author");
    Ok(crate::project::init(name, prefix, author)?)
}

#[cfg(windows)]
fn ansi_support() {
    // Attempt to enable ANSI support in terminal
    // Disable colored output if failed
    if !ansi_term::enable_ansi_support().is_ok() {
        colored::control::set_override(false);
    }
}

#[cfg(not(windows))]
fn ansi_support() {
    unreachable!();
}
