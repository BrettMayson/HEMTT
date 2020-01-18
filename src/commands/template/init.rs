use crate::{HEMTTError, Project};

pub fn run(p: Project) -> Result<(), HEMTTError> {
    let items = ["CBA", "ACE", "Vanilla", "Custom"];
    let selection = {
        let mut select = dialoguer::Select::new();
        select.default(0);
        select.items(&items);
        select.interact_opt()?
    };
    if selection.is_none() {
        unimplemented!()
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
    Ok(())
}
