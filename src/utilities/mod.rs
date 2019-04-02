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
    Translation,
    Zip,
}
impl FromStr for Utility {
    type Err = ();
    fn from_str(s: &str) -> Result<Utility, ()> {
        match s.to_lowercase().as_str() {
            "armake" => Ok(Utility::Armake),
            "convertproject" => Ok(Utility::ConvertProject),
            "translation" => Ok(Utility::Translation),
            "zip" => Ok(Utility::Zip),
            _ => Err(()),
        }
    }
}

pub fn find(utility: &str) -> Option<Utility> {
    return match utility {
        "armake" => Some(Utility::Armake),
        "convertproject" => Some(Utility::ConvertProject),
        "translation" => Some(Utility::Translation),
        "zip" => Some(Utility::Zip),
        _ => None
    }
}

pub fn run(utility: &Utility, args: &Vec<String>) -> Result<(), std::io::Error> {
    #[allow(unreachable_patterns)]
    return match utility {
        Utility::Armake => armake::run(args),
        Utility::ConvertProject => convert::run(),
        Utility::Translation => translation::check(),
        Utility::Zip => zip::archive(args),
        _ => Err(error!("Utility not implemented"))
    }
}
