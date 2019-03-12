use armake2::preprocess::*;
use armake2::io::{Input,Output}; // I definitely shouldn't be using this
// use pbr::ProgressBar;
use rayon::prelude::*;
use rayon::iter::ParallelBridge;
use walkdir::WalkDir;

use std::fs::File;
// use std::io::BufReader;
use std::io::{stdout,Read,Write};
use std::path::PathBuf;

use crate::error::*;
use crate::project::*;

pub fn show() -> Result<(), std::io::Error> {
    let mut p = crate::project::get_project().unwrap();
    let configs: Vec<PathBuf> = WalkDir::new("addons").into_iter().map(|e| e.unwrap_or_print().path().to_path_buf()).filter(|e| e.ends_with("config.cpp")).collect();
    p.include.push(PathBuf::from("."));
    // This is slow enough that we probably need a progress bar :(
    let vecconf = configs.par_iter().map(|config| {
        // turn this into a .map eventually
        // this obviously shouldn't output to stdout
        let mut buffer = String::new();
        File::open(config).unwrap().read_to_string(&mut buffer).expect("Failed to read config"); // TODO: improve error
        return armake2::preprocess::preprocess(buffer, Some(config.to_path_buf()),&p.include).unwrap().0 // Magic number here accesses the actual string rather than "preprocess info"

        // really, we want to be able to tell preprocess to stop once it 
        // gets to a line matching certain criteria
        // i.e, we want to parse up to requiredAddons[] and no further
        // https://github.com/KoffeinFlummi/armake2/blob/40fabd915514ffda372ec012b35ed4190d0e0515/src/preprocess.rs#L369
        // but that's just perf so let's leave it for now
    });
    let mut file = File::create("test.txt")?;
    let confs: Vec<String> = vecconf.collect();
    file.write(confs.join("\n").as_bytes());
    Ok(())
}
