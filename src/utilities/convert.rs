use std::fs;
use std::fs::File;

use crate::error::*;

pub fn run() -> Result<(), std::io::Error> {
    crate::check(false, false)?;
    let p = crate::project::get_project()?;
    let file = crate::project::path(false).unwrap_or_print();
    match file.extension().unwrap().to_str().unwrap() {
        "toml" => {
            remove_file!("hemtt.toml")?;
            create_file!("hemtt.json")?;
        },
        "json" => {
            remove_file!("hemtt.json")?;
            create_file!("hemtt.toml")?;
        },
        _ => unreachable!()
    }
    p.save()
}
