use std::path::PathBuf;

use crate::{HEMTTError, Project};

pub fn run(_: Project) -> Result<(), HEMTTError> {
    let items = ["CBA", "ACE", "Vanilla", "Custom"];
    let selection = {
        let mut select = dialoguer::Select::new();
        select.default(0);
        select.items(&items);
        select.interact_opt()?
    };
    if selection.is_none() {
        warn!("Template init cancelled");
        return Ok(());
    }
    let selection = selection.unwrap();
    let selection = if selection == items.len() - 1 {
        ask!("Template URL")?
    } else {
        items[selection].to_string()
    };
    // clone template
    let repo = if selection.starts_with("http") {
        selection
    } else {
        format!("https://github.com/hemtt/{}", selection)
    };
    match git2::Repository::clone(&repo, "./.hemtt/template") {
        Ok(_) => println!("Template Cloned"),
        Err(e) => panic!("Failed to clone: {}", e),
    };

    let init_folder = PathBuf::from(".hemtt/template/init/.");
    if init_folder.exists() {
        println!("Initilizing Template");
        // TODO handle error
        let mut options = fs_extra::dir::CopyOptions::new();
        options.copy_inside = true;
        let entries = std::fs::read_dir(init_folder)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;
        fs_extra::copy_items(&entries, PathBuf::from("."), &options);
    } else {
        warn!("Template does not contain an init folder");
    }

    Ok(())
}
