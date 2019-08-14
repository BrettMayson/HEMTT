use colored::*;
use docopt::Docopt;
use num_cpus;
use self_update;
use serde::Deserialize;

#[macro_use]
pub mod macros;

#[cfg(windows)]
use ansi_term;

use std::collections::{HashSet};
use std::fs;
use std::io::{stdin, stdout, Write, Error};
use std::path::{Path, PathBuf};
use std::str::FromStr;

mod build;
mod error;
mod files;
mod helpers;
mod project;
mod state;
mod template;
mod utilities;

use crate::error::*;
use crate::utilities::Utility;

#[allow(non_snake_case)]
#[cfg(debug_assertions)]
fn VERSION() -> String {
    format!("{}-debug", env!("CARGO_PKG_VERSION"))
}

#[allow(non_snake_case)]
#[cfg(not(debug_assertions))]
fn VERSION() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

const USAGE: &str = "
HEMTT, a simple to use build manager for Arma 3 mods using the CBA project structure

Usage:
    hemtt init
    hemtt create
    hemtt addon <name>
    hemtt build [<addons>] [--release] [--force] [--nowarn] [--opts=<addons>] [--skip=<addons>] [--jobs=<n>]
    hemtt clean [--force]
    hemtt run <script>
    hemtt update
    hemtt (-h | --help)
    hemtt --version

Commands:
    init                Initialize a project file in the current directory
    create              Create a new project using the CBA project structure
    addon               Create a new addon folder
    build               Build the project
    clean               Clean build files
    update              Update HEMTT

Utilities:
    armake              Run armake2 commands
    convertproject      Convert project file between JSON and TOML
    translation         Displays the translation progress of all stringtable files
    zip                 Create a .zip of the latest release

Options:
    -v --verbose        Enable verbose output
    -f --force          Overwrite target files
       --nowarn         Suppress armake2 warnings
       --opts=<addons>  Comma seperated list of addtional components to build
       --skip=<addons>  Comma seperated list of addons to skip building
    -j --jobs=<n>       Number of parallel jobs, defaults to # of CPUs
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
    cmd_run: bool,
    cmd_update: bool,
    flag_verbose: bool,
    flag_force: bool,
    flag_nowarn: bool,
    flag_version: bool,
    flag_release: bool,
    flag_opts: String,
    flag_skip: String,
    flag_jobs: usize,
    arg_script: String,
    arg_name: String,
    arg_addons: String,
}

fn input(text: &str, default: Option<String>) -> String {
    let mut s = String::new();
    let ret = match default {
        Some(v) => {
            print!("{} ({}): ", text, &v);
            v
        }
        None => {
            print!("{}: ", text);
            String::new()
        }
    };
    stdout().flush().unwrap();
    stdin().read_line(&mut s).expect("Did not enter a valid string");
    if let Some('\n')=s.chars().next_back() {
        s.pop();
    }
    if let Some('\r')=s.chars().next_back() {
        s.pop();
    }
    if s.is_empty() {
        return ret;
    }
    s
}

fn run_command(args: &Args) -> Result<(), Error> {
    if args.cmd_init {
        check(true, args.flag_force).unwrap_or_print();
        init().unwrap();
        Ok(())
    } else if args.cmd_create {
        if Path::new("addons").exists() {
            return Err(error!("The current directory already has a mod. Use init instead of create."));
        }
        check(true, args.flag_force).unwrap_or_print();
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
        check(false, args.flag_force).unwrap_or_print();
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
        check(false, args.flag_force).unwrap_or_print();
        let p = project::get_project().unwrap();
        if !args.flag_nowarn {
            unsafe {
                armake2::error::WARNINGS_MUTED = Some(HashSet::new());
            }
        }
        let mut addons: Vec<PathBuf>  = Vec::new();
        let mut skip: Vec<String> = p.skip.clone();
        let version = match &p.version {
            Some(v) => v,
            None => panic!("Unable to determine version number"),
        };
        if args.flag_release {
            if args.flag_force {
                files::clear_release(&p, &version).unwrap();
                let mut pbos: Vec<PathBuf> = fs::read_dir("addons").unwrap()
                    .map(|file| file.unwrap().path())
                    .filter(|file_or_dir| file_or_dir.is_dir())
                    .collect();
                if Path::new("optionals/").exists() {
                    let optionals: Vec<PathBuf> = fs::read_dir("optionals").unwrap()
                        .map(|file| file.unwrap().path())
                        .filter(|file_or_dir| file_or_dir.is_dir())
                        .collect();
                    pbos.append(&mut optionals.clone());
                }
                files::clear_pbos(&p, &pbos).unwrap();
            }
            println!(" {} release v{}", "Preparing".green().bold(), version);
            if Path::new(&format!("releases/{}", version)).exists() {
                return Err(error!("Release already exists, run with --force to clean"));
            }
        }

        if args.flag_skip != "" {
            let mut specified_skip: Vec<String> = args.flag_skip.split(',').map(|s| s.to_string()).collect();
            skip.append(&mut specified_skip);
            skip.sort();
            skip.dedup();
        }
        if args.flag_opts == "all" {
            for entry in fs::read_dir("optionals")? {
                let entry = entry.unwrap();
                if !entry.path().is_dir() { continue };
                if skip.contains(&entry.path().file_name().unwrap().to_str().unwrap().to_owned()) { continue };
                addons.push(entry.path());
            }
        } else if args.flag_opts != "" {
            let specified_optionals: Vec<String> = args.flag_opts.split(',').map(|s| s.to_string()).collect();
            let optional_path = PathBuf::from("optionals");
            for optional in specified_optionals {
                let mut opt = optional_path.clone();
                opt.push(&optional);
                if !opt.exists() {
                    return Err(error!("optionals/{} was not found", optional.bold()));
                }
                if skip.contains(&optional) { continue };
                addons.push(opt);
            }
            for optional in &p.optionals
            {
                let mut opt = optional_path.clone();
                if skip.contains(&optional) { continue };
                opt.push(optional);
                addons.push(opt);
            }
        } else if args.flag_release && Path::new("optionals/").exists() {
            for entry in fs::read_dir("optionals")? {
                let entry = entry.unwrap();
                if !entry.path().is_dir() { continue };
                if skip.contains(&entry.path().file_name().unwrap().to_str().unwrap().to_owned()) { continue };
                addons.push(entry.path());
            }
        }
        if args.arg_addons != "" {
            let specified_addons: Vec<String> = args.arg_addons.split(',').map(|s| s.to_string()).collect();
            let addon_path = PathBuf::from("addons");
            for addon in specified_addons {
                let mut adn = addon_path.clone();
                adn.push(&addon);
                if !adn.exists() {
                    return Err(error!("addons/{} was not found", addon.bold()));
                }
                if skip.contains(&addon) { continue };
                addons.push(adn);
            }
        } else {
            for addon in crate::files::all_addons() {
                if skip.contains(&addon.file_name().unwrap().to_str().unwrap().to_owned()) { continue };
                addons.push(addon);
            }
        }
        if args.flag_force {
            for addon in &addons {
                if !skip.contains(&addon.file_name().unwrap().to_str().unwrap().to_owned()) {
                    crate::files::clear_pbo(&p, &addon).unwrap_or_print();
                }
            }
        }
        let mut state = crate::state::State::new(&addons);
        p.run(&state).unwrap_or_print();
        let result = build::addons(&p, &addons).unwrap_or_print();
        state.stage = crate::state::Stage::PostBuild;
        state.result = Some(&result);
        p.run(&state).unwrap_or_print();
        if args.flag_release {
            build::release::release(&p, &version).unwrap_or_print();
            state.stage = crate::state::Stage::ReleaseBuild;
            p.run(&state).unwrap_or_print();
            println!("  {} {} v{}", match result.failed.len() {
                0 => "Finished".green().bold(),
                _ => "Finished".yellow().bold(),
             }, &p.name, version);
        } else {
            println!("  {} {}", match result.failed.len() {
                0 => "Finished".green().bold(),
                _ => "Finished".yellow().bold(),
             }, &p.name);
        }
        if !args.flag_nowarn {
            armake2::error::print_warning_summary();
        }
        if !result.failed.is_empty() {
            return Err(error!("Building of at least one addon failed"));
        }
        Ok(())
    } else if args.cmd_clean {
        check(false, args.flag_force).unwrap_or_print();
        let p = project::get_project().unwrap();
        let mut pbos: Vec<PathBuf> = fs::read_dir("addons").unwrap()
            .map(|file| file.unwrap().path())
            .filter(|file_or_dir| file_or_dir.is_dir())
            .collect();
        if Path::new("optionals/").exists() {
            let optionals: Vec<PathBuf> = fs::read_dir("optionals").unwrap()
                .map(|file| file.unwrap().path())
                .filter(|file_or_dir| file_or_dir.is_dir())
                .collect();
            pbos.append(&mut optionals.clone());
        }
        files::clear_pbos(&p, &pbos).unwrap_or_print();
        if args.flag_force {
            files::clear_releases(&p).unwrap_or_print();
        }
        Ok(())
    } else if args.cmd_run {
        check(false, args.flag_force).unwrap_or_print();
        let addons = Vec::new();
        let mut state = crate::state::State::new(&addons);
        state.stage = crate::state::Stage::Script;
        let p = project::get_project().unwrap();
        p.script(&args.arg_script, &state).unwrap_or_print();
        Ok(())
    } else if args.cmd_update {
        let target = self_update::get_target();
        let status = self_update::backends::github::Update::configure()
            .repo_owner("SynixeBrett")
            .repo_name("HEMTT")
            .target(&target)
            .bin_name(if cfg!(windows) {"hemtt.exe"} else {"hemtt"})
            .show_download_progress(true)
            .current_version(&VERSION())
            .build().unwrap()
            .update().unwrap();
        println!("Using Version: {}", status.version());
        Ok(())
    } else {
        Ok(())
    }
}

fn main() {
    if cfg!(windows) {
        ansi_support();
    }

    let mut args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| {
            let mut args = std::env::args().collect::<Vec<_>>();
            args.remove(0);

            if args.len() == 0 {
                // No arguments provided, show usage
                e.exit();
            }

            let utility = Utility::from_str(&args[0]);
            if utility.is_ok() {
                utilities::run(&utility.unwrap(), &mut args).unwrap_or_print();
                std::process::exit(0);
            }
            e.exit();
        });

    if args.flag_version {
        println!("HEMTT Version {}", &VERSION());
        std::process::exit(0);
    }

    if args.flag_jobs == 0 {
        args.flag_jobs = num_cpus::get();
    }
    rayon::ThreadPoolBuilder::new().num_threads(args.flag_jobs).build_global().unwrap();

    run_command(&args).unwrap_or_print();
}

fn check(write: bool, force: bool) -> Result<(), Error> {
    let exists = crate::project::exists().is_ok();
    if exists && write && !force {
        Err(error!("HEMTT Project already exists in the current directory"))
    } else if exists && write && force {
        Ok(())
    } else if !exists && !write {
        Err(error!("A HEMTT Project does not exist in the current directory"))
    } else {
        Ok(())
    }
}

fn init() -> Result<crate::project::Project, Error> {
    let name = input("Project Name", Some("My Cool Mod".to_owned()));
    let prefix = input("Prefix", Some("MCM".to_owned()));
    let author = input("Author", Some("Me".to_owned()));
    Ok(crate::project::init(name, prefix, author)?)
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

fn is_true(v: &bool) -> bool { v.clone() }
fn is_false(v: &bool) -> bool { !v.clone() }
fn dft_true() -> bool { true }
fn dft_false() -> bool { false }
