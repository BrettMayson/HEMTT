use std::fs;
use std::fs::File;

pub fn run() -> Result<(), std::io::Error> {
    crate::check(false, false)?;
    let p = crate::project::get_project()?;
    if !crate::project::toml_exists() {
        // Convert to TOML
        if crate::project::json_exists() {
            fs::remove_file("hemtt.json")?;
        }
        File::create("hemtt.toml")?;
    } else {
        // Convert to JSON
        fs::remove_file("hemtt.toml")?;
        File::create("hemtt.json")?;
    }
    p.save()
}
