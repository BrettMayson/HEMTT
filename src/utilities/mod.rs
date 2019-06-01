use serde::Deserialize;

use std::str::FromStr;

pub mod convert;
pub mod convert_project;
pub mod translation;
pub mod zip;
pub mod armake;

#[derive(Debug, Deserialize)]
pub enum Utility {
    Armake,
    Convert,
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
            "convert" => Ok(Utility::Convert),
            "convertproject" => Ok(Utility::ConvertProject),
            "translation" => Ok(Utility::Translation),
            "zip" => Ok(Utility::Zip),
            _ => Err(()),
        }
    }
}

pub fn find(utility: &str) -> Option<Utility> {
    return match Utility::from_str(utility) {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

pub fn run(utility: &Utility, args: &mut Vec<String>) -> Result<(), std::io::Error> {
    #[allow(unreachable_patterns)]
    return match utility {
        Utility::Armake => armake::run(args),
        Utility::Convert => convert::run(args),
        Utility::ConvertProject => convert_project::run(),
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
