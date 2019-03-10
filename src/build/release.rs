use colored::*;
use rayon::prelude::*;

use std::fs;
use std::io::{Error};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub fn release(p: &crate::project::Project, version: &String) -> Result<(), Error> {
    let modname = p.get_modname();
    if !Path::new(&format!("releases/{}/@{}/addons", version, modname)).exists() {
        fs::create_dir_all(format!("releases/{}/@{}/addons", version, modname))?;
    }
    if !Path::new(&format!("releases/{}/@{}/keys", version, modname)).exists() {
        fs::create_dir_all(format!("releases/{}/@{}/keys", version, modname))?;
    }
    for file in &p.files {
        fs::copy(file, format!("releases/{}/@{}/{}", version, modname, file))?;
    }

    // Generate key
    if !Path::new("releases/keys").exists() {
        fs::create_dir("releases/keys")?;
    }
    let keyname = p.get_keyname();
    if !Path::new(&format!("releases/keys/{}.bikey", keyname)).exists() {
        println!("    {} {}.bikey", "KeyGen".green().bold(), keyname);
        armake2::sign::cmd_keygen(PathBuf::from(&keyname))?;
        fs::rename(format!("{}.bikey", keyname), format!("releases/keys/{}.bikey", keyname))?;
        fs::rename(format!("{}.biprivatekey", keyname), format!("releases/keys/{}.biprivatekey", keyname))?;
    }

    // Sign
    fs::copy(format!("releases/keys/{}.bikey", keyname), format!("releases/{}/@{}/keys/{}.bikey", version, modname, keyname))?;

    let count = Arc::new(Mutex::new(0));

    let mut folder = String::from("addons");
    let dirs: Vec<_> = fs::read_dir(&folder).unwrap()
        .map(|file| file.unwrap())
        .collect();
    dirs.par_iter().for_each(|entry| {
        // TODO split copy and sign
        if crate::build::sign::copy_sign(&folder, &entry, &p, &version).unwrap() {
            *count.lock().unwrap() += 1;
        }
    });

    folder = String::from("optionals");
    if Path::new(&folder).exists() {
        if !Path::new(&format!("releases/{}/@{}/{}", version, modname, folder)).exists() {
            fs::create_dir_all(format!("releases/{}/@{}/{}", version, modname, folder))?;
        }
        let opts: Vec<_> = fs::read_dir(&folder).unwrap()
            .map(|file| file.unwrap())
            .collect();
        opts.par_iter().for_each(|entry| {
            // TODO split copy and sign
            if crate::build::sign::copy_sign(&folder, &entry, &p, &version).unwrap() {
                *count.lock().unwrap() += 1;
            }
        });
    }

    println!("    {} {}", "Signed".green().bold(), *count.lock().unwrap());
    Ok(())
}
