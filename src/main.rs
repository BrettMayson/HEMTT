use serde::Deserialize;
use docopt::Docopt;

use self_update;

use std::io::{stdin, stdout, Write};
use std::path::Path;

mod project;
mod files;
mod build;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const HEMTT_FILE: &str = "hemtt.json";

const USAGE: &'static str = "
HEMTT, a simple to use build manager for Arma 3 mods using CBA

Usage:
  hemtt init
  hemtt create
  hemtt addon <name>
  hemtt build
  hemtt update
  hemtt (-h | --help)
  hemtt --version

Commands:
  init        Initialize a project file in the current directory
  create      Create a new project from the CBA project template
  addon       Create a new addon folder
  build       Build the project
  update      Update HEMTT

Options:
  -v --verbose        Enable verbose output
  -f --force          Overwrite target files
  -h --help           Show usage information and exit
     --version        Show version number and exit
";

#[derive(Debug, Deserialize)]
struct Args {
  cmd_init: bool,
  cmd_create: bool,
  cmd_addon: bool,
  cmd_build: bool,
  cmd_update: bool,
  flag_verbose: bool,
  flag_force: bool,
  flag_version: bool,
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

// TODO move code in main into an error wrapper
fn main() {
  let args: Args = Docopt::new(USAGE)
                           .and_then(|d| d.deserialize())
                           .unwrap_or_else(|e| e.exit());

  if args.flag_version {
    println!("HEMTT Version {}", VERSION);
    std::process::exit(0);
  }

  if args.cmd_init {
    check(true, args.flag_force).unwrap();
    init().unwrap();
  } else if args.cmd_create {
    check(true, args.flag_force).unwrap();
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
  } else if args.cmd_addon {
    check(false, args.flag_force).unwrap();
    let p = project::get_project().unwrap();
    if Path::new(&format!("addons/{}", args.arg_name)).exists() {
      println!("Addon {} already exists!", args.arg_name);
      return;
    }
    println!("Creating addon: {}", args.arg_name);
    files::create_addon(&args.arg_name, &p).unwrap();
    files::pboprefix(&args.arg_name, &p).unwrap();
    files::script_component(&args.arg_name, &p).unwrap();
    files::configcpp(&args.arg_name, &p).unwrap();
    files::xeh(&args.arg_name, &p).unwrap();
  } else if args.cmd_build {
    check(false, args.flag_force).unwrap();
    let p = project::get_project().unwrap();
    if args.flag_force {
      files::clear_pbos(&p).unwrap();
    }
    build::build(&p).unwrap();
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
    println!("Update: {}", status.version());
  }
}

fn check(write: bool, force: bool) -> Result<(), std::io::Error> {
  if Path::new(HEMTT_FILE).exists() && write && !force {
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "HEMTT Project already exists in the current directory".to_owned()
    ))
  } else if Path::new(HEMTT_FILE).exists() && write && force {
    Ok(())
  } else if !Path::new(HEMTT_FILE).exists() && !write {
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "A HEMTT Project does not exist in the current directory".to_owned()
    ))
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