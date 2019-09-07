use serde::Deserialize;

use std::str::FromStr;

pub mod convert;
pub mod translation;
pub mod zip;
pub mod armake;

#[derive(Debug, Deserialize)]
pub enum Utility {
    Armake,
    ConvertProject,
    Template,
    Translation,
    Zip,
}
impl FromStr for Utility {
    type Err = ();
    fn from_str(s: &str) -> Result<Utility, ()> {
        match s.to_lowercase().as_str() {
            "armake" => Ok(Utility::Armake),
            "convertproject" => Ok(Utility::ConvertProject),
            "template" => Some(Utility::Template),
            "translation" => Ok(Utility::Translation),
            "zip" => Ok(Utility::Zip),
            _ => Err(()),
        }
    }
}

pub fn find(utility: &str) -> Option<Utility> {
    let name = Utility::from_str(utility);
    if name.is_ok() {
        Some(name.unwrap())
    } else {
        None   
    }
}

pub fn run(utility: &Utility, args: &mut Vec<String>) -> Result<(), std::io::Error> {
    #[allow(unreachable_patterns)]
    match utility {
        Utility::Armake => armake::run(args),
        Utility::ConvertProject => convert::run(),
        Utility::Template => {
            let p = crate::project::get_project()?;
            args.remove(0);
            println!("{}", p.render(&args.join(" ").to_owned()));
            Ok(())
        },
        Utility::Translation => translation::check(),
        Utility::Zip => zip::archive(args),
        _ => Err(error!("Utility not implemented"))
    }
}
