use armake2::{pbo::PBO, sign::BISignVersion};
use colored::*;

use std::fs;
use std::fs::File;
use std::io::Error;
use std::path::PathBuf;

pub fn copy(path_posix: &String, addonsfolder: &String, pbo_filename: &String) -> Result<bool, Error> {
    fs::copy(path_posix, format!("{}/{}", addonsfolder, pbo_filename))?;
    Ok(true)
}

pub fn sign_version(v: u8) -> BISignVersion {
    match v {
        3 => BISignVersion::V3,
        2 => BISignVersion::V2,
        _ => {
            yellow!("KeyWarn", format!("Invalid Version {}", v));
            yellow!("KeyWarn", format!("Using V{}", crate::project::dft_sig()));
            sign_version(crate::project::dft_sig())
        }
    }
}

pub fn sign(
    pbo_filename: &String,
    addonsfolder: &String,
    p: &crate::project::Project,
    key: &armake2::sign::BIPrivateKey,
) -> Result<bool, Error> {
    let pbo =
        PBO::read(&mut File::open(PathBuf::from(format!("{}/{}", addonsfolder, pbo_filename))).expect("Failed to open PBO"))
            .expect("Failed to read PBO");

    let sig = key.sign(&pbo, sign_version(p.sigversion));
    let signame = p.get_signame(&pbo_filename);
    sig.write(&mut File::create(PathBuf::from(format!("{}/{}", addonsfolder, signame))).unwrap())?;

    Ok(true)
}

pub fn copy_sign(
    folder: &String,
    path: &PathBuf,
    p: &crate::project::Project,
    key: &armake2::sign::BIPrivateKey,
) -> Result<bool, Error> {
    let path_posix = path.clone().to_str().unwrap().replace(r#"\"#, "/");
    let pbo_filename = path.file_name().unwrap().to_str().unwrap().to_string();

    if !path.ends_with(".pbo") && !pbo_filename.starts_with(p.prefix.as_str()) {
        return Ok(false);
    }

    let modname = p.get_modname();
    let ver = p.version.clone().unwrap();

    let addonsfolder = iformat!("releases/{ver}/@{modname}/{folder}", ver, modname, folder);

    copy(&path_posix, &addonsfolder, &pbo_filename)?;

    sign(&pbo_filename, &addonsfolder, p, key)?;

    Ok(true)
}
