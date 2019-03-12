use armake2::preprocess::*;
use armake2::io::{Input,Output}; // I definitely shouldn't be using this
// use pbr::ProgressBar;
use rayon::prelude::*;
use walkdir::WalkDir;

use std::fs::File;
// use std::io::BufReader;
use std::io::stdout;
use std::path::PathBuf;

use crate::error::*;
use crate::project::*;

pub fn show() -> Result<(), std::io::Error> {
    let mut p = crate::project::get_project().unwrap();
    let configs = WalkDir::new("addons").into_iter().map(|e| e.unwrap_or_print()).filter(|e| e.path().to_path_buf().ends_with("config.cpp"));
    // configs.par_iter().for_each(|config| {
    // this don't work for some reason ^
    // figure out includes?
    p.include.push(PathBuf::from("."));
    configs.for_each(|config| {
        // turn this into a .map eventually
        // this obviously shouldn't output to stdout
        armake2::preprocess::cmd_preprocess(&mut File::open(config.path().to_path_buf()).unwrap(), &mut Output::Standard(stdout()),Some(config.path().to_path_buf()),&p.include);
        // Weirdly, this works, but the other thing doesn't
        //armake2::preprocess::cmd_preprocess(&mut File::open(config.path().to_path_buf()).unwrap(), &mut Output::Standard(stdout()),Some(path),&p.include);
    });
    Ok(())
}
