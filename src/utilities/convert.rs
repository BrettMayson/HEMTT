use std::fs;
use std::fs::File;

use crate::error::*;

pub fn run() -> Result<(), std::io::Error> {
    crate::check(false, false)?;
    let p = crate::project::get_project()?;
    let file = crate::project::path(false).unwrap_or_print();
    match file.extension().unwrap().to_str().unwrap() {
        "toml" => {
            fs::remove_file("hemtt.toml")?;
            File::create("hemtt.json")?;
        },
        "json" => {
            fs::remove_file("hemtt.json")?;
            File::create("hemtt.toml")?;
        },
        _ => unreachable!()
    }
    p.save()
}
