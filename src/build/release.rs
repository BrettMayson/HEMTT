use armake2::sign::{cmd_keygen, BIPrivateKey};
use colored::*;
use glob::glob;
use rayon::prelude::*;

use std::fs;
use std::fs::File;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::error::*;
use crate::build::sign;

pub fn release(p: &crate::project::Project, version: &String) -> Result<(), Error> {
    let modname = p.get_modname();
    if !Path::new(&format!("releases/{}/@{}/addons", version, modname)).exists() {
        fs::create_dir_all(format!("releases/{}/@{}/addons", version, modname))?;
    }
    if !Path::new(&format!("releases/{}/@{}/keys", version, modname)).exists() {
        fs::create_dir_all(format!("releases/{}/@{}/keys", version, modname))?;
    }
    for file in &p.files {
        for entry in glob(file).unwrap_or_print() {
            if let Ok(path) = entry {
                let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
                fs::copy(&path, format!("releases/{}/@{}/{}", version, modname, file_name))?;
            }
        }
    }

    if !Path::new("releases/keys").exists() {
        fs::create_dir("releases/keys")?;
    }

    let mut key;
    let keyname = p.get_keyname();
    // Generate and store key if required
    if p.reuse_private_key {

        // Make a new keypair if there isn't one already
        if !Path::new(&format!("releases/keys/{}.bikey", keyname)).exists() {
            println!("    {} {}.bikey", "KeyGen".green().bold(), keyname);

            // Generate and write the keypair to disk in the current directory
            cmd_keygen(PathBuf::from(&keyname))?;
            fs::rename(
                format!("{}.bikey", keyname),
                format!("releases/keys/{}.bikey", keyname),
            )?;
            fs::rename(
                format!("{}.biprivatekey", keyname),
                format!("releases/keys/{}.biprivatekey", keyname),
            )?;
        }

        // Read the private key from disk
        key = BIPrivateKey::read(
            &mut File::open(format!("releases/keys/{}.biprivatekey", keyname))
                .expect("Failed to open private key"),
        )
        .expect("Failed to read private key");
    } else {
        // Make the private key and leave it in memory
        key = BIPrivateKey::generate(1024, keyname.clone());
        let public_key = key.to_public_key();

        // Write the public key to disk
        public_key.write(
            &mut std::fs::File::create(format!("releases/keys/{}.bikey", keyname)).unwrap_or_print(),
        )?;
    }

    // Copy public key to specific release dir
    fs::copy(
        format!("releases/keys/{}.bikey", keyname),
        format!("releases/{}/@{}/keys/{}.bikey", version, modname, keyname),
    )?;

    let count = Arc::new(Mutex::new(0));

    // Sign
    let mut folder = String::from("addons");
    let dirs: Vec<_> = fs::read_dir(&folder)
        .unwrap_or_print()
        .map(|file| file.unwrap_or_print())
        .collect();
    dirs.par_iter().for_each(|entry| {
        // TODO split copy and sign
        if sign::copy_sign(&folder, &entry.path(), &p, &key).unwrap_or_print() {
            *count.lock().unwrap_or_print() += 1;
        }
    });

    folder = String::from("optionals");
    if Path::new(&folder).exists() {
        let addonsfolder = format!(
            "releases/{ver}/@{mod}/{addons}", ver=version, mod=modname, addons=folder
        );
        if !Path::new(&addonsfolder).exists() {
            fs::create_dir_all(addonsfolder)?;
        }
        let opts: Vec<_> = fs::read_dir(&folder)
            .unwrap_or_print()
            .map(|file| file.unwrap_or_print())
            .collect();
        opts.par_iter().for_each(|entry| {
            // TODO split copy and sign
            // for copying, we need to know source path, addons folder and pbo_filename (we could
            // get this but that seems like extra faff)
            // for signing, we need to know addons folder, PBO file name and key
            if sign::copy_sign(&folder, &entry.path(), &p, &key).unwrap_or_print() {
                *count.lock().unwrap_or_print() += 1;
            }
        });
    }

    green!("Signed", *count.lock().unwrap_or_print());
    Ok(())
}
