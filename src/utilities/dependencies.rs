use armake2::preprocess::*;
use armake2::io::{Input,Output}; // I definitely shouldn't be using this
// use pbr::ProgressBar;
use petgraph::Graph;
use rayon::prelude::*;
use rayon::iter::ParallelBridge;
use regex::Regex;
use walkdir::WalkDir;

use std::fs::File;
// use std::io::BufReader;
use std::io::{stdout,Read,Write};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::*;
use crate::project::*;

pub fn show() -> Result<(), std::io::Error> {
    let mut p = crate::project::get_project().unwrap();
    let configs: Vec<PathBuf> = WalkDir::new("addons").into_iter().map(|e| e.unwrap_or_print().path().to_path_buf()).filter(|e| e.ends_with("config.cpp")).collect();
    p.include.push(PathBuf::from("."));

    let gimmeCfgPatch = Regex::new(r"(?i)class CfgPatches").unwrap();
    let gimmeClass = Regex::new(r"(?i)class ([^{]*)").unwrap();
    let gimmeReqs = Regex::new(r"(?i)requiredAddons.*\{(.*)\}").unwrap();

    // This is slow enough that we probably need a progress bar :(
    let mut deps = Arc::new(Mutex::new(Graph::<_, ()>::new()));
    let mut name2node = Arc::new(Mutex::new(HashMap::new()));
    let vecconf = configs.par_iter().map(|config| {
        // turn this into a .map eventually
        // this obviously shouldn't output to stdout
        let mut buffer = String::new();
        File::open(config).unwrap().read_to_string(&mut buffer).expect("Failed to read config"); // TODO: improve error
        // Magic number here accesses the actual string rather than "preprocess info"
        let processed = armake2::preprocess::preprocess(buffer, Some(config.to_path_buf()),&p.include).unwrap().0;
        let processed_lines = processed.split('\n');
        let mut looking_for = "patches";
        let mut name = "";
        for line in processed_lines {
            if ((looking_for == "reqs") && gimmeReqs.is_match(line)) {
                // Zeroth match corresponds to whole line - first match is first capture group
                println!("{}",gimmeReqs.captures(line).unwrap().get(1).unwrap().as_str()); 
                gimmeReqs.captures(line).unwrap().get(1).unwrap().as_str().split(',').map(|s| s.replace('"',"")).for_each(|req| {
                    if !name2node.lock().unwrap().contains_key(&req){
                        let nodenum = deps.lock().unwrap().add_node(req.to_string());
                        name2node.lock().unwrap().insert(req.to_string(),nodenum);
                    }
                    // TODO
                    // now make edges each dep to name
                    // deps.add_edge(hashmap.getval(dep),hashmap.getval(name))
                    // owtte
                });
                break;
                // looking_for = "patches"; // TODO - handle multi-line matches (!)
            }
            if ((looking_for == "class") && gimmeClass.is_match(line)) {
                // Zeroth match corresponds to whole line - first match is first capture group
                name = gimmeClass.captures(line).unwrap().get(1).unwrap().as_str() ;
                println!("{}",name);
                if !name2node.lock().unwrap().contains_key(name){
                    let nodenum = deps.lock().unwrap().add_node(name.to_string());
                    name2node.lock().unwrap().insert(name.to_string(),nodenum);
                }
                looking_for = "reqs"; // TODO - handle reqs not existing
            }
            if gimmeCfgPatch.is_match(line) {
                looking_for = "class";
            }
        }
        // This seems to print stuff fine
        for (key, value) in &*name2node.lock().unwrap() {
            println!("{} -> {}", key, value.index());
        }
        return processed;


        // really, we want to be able to tell preprocess to stop once it 
        // gets to a line matching certain criteria
        // i.e, we want to parse up to requiredAddons[] and no further
        // https://github.com/KoffeinFlummi/armake2/blob/40fabd915514ffda372ec012b35ed4190d0e0515/src/preprocess.rs#L369
        // but that's just perf so let's leave it for now
    });
    // This prints first? I guess we need to wait for the loop to finish or something weird?
    println!("the last line");
    // This doesn't print anything at all :/
    for (key, value) in &*name2node.lock().unwrap() {
        println!("{} -> {}", key, value.index());
    }
    let mut file = File::create("test.txt")?;
    let confs: Vec<String> = vecconf.collect();
    file.write(confs.join("\n").as_bytes());
    Ok(())
}
