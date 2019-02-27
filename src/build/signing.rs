use std::io::{Error};
use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;

use crate::error::*;

pub fn copy_sign(folder: &String, entry: &DirEntry, p: &crate::project::Project, version: &String) -> Result<bool, Error> {
    let path = entry.path();
    let cpath = path.clone();
    let cpath = cpath.to_str().unwrap().replace(r#"\"#,"/");
    let pbo = cpath.replace((folder.clone() + "/").as_str(), "");
    if !path.ends_with(".pbo") && !pbo.contains(p.prefix.as_str()) {
        return Ok(false);
    }

    let modname = p.get_modname();
    fs::copy(&cpath, format!("releases/{}/@{}/{}/{}", version, modname, folder, pbo))?;

    let signame = p.get_signame(&pbo);
    let keyname = p.get_keyname();

    armake2::sign::cmd_sign(
        PathBuf::from(format!("releases/keys/{}.biprivatekey", keyname)),
        PathBuf::from(format!("releases/{}/@{}/{}/{}", version, modname, folder, pbo)),
        Some(PathBuf::from(format!("releases/{0}/@{1}/{2}/{3}", version, modname, folder, signame))),
        armake2::sign::BISignVersion::V3
    ).print_error(false);
    Ok(true)
}
