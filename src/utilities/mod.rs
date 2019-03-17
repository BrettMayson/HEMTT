use serde::Deserialize;

use std::io::{Error};

pub mod convert;
pub mod translation;
pub mod zip;

#[derive(Debug, Deserialize)]
pub enum Utility {
    ConvertProject,
    Translation,
    Zip
}

pub fn find(utility: &str) -> Option<Utility> {
    return match utility {
        "convertproject" => Some(Utility::ConvertProject),
        "translation" => Some(Utility::Translation),
        "zip" => Some(Utility::Zip),
        _ => None
    }
}

pub fn run(utility: &Utility, args: &Vec<String>) -> Result<(), std::io::Error> {
    #[allow(unreachable_patterns)]
    return match utility {
        Utility::ConvertProject => convert::run(),
        Utility::Translation => translation::check(),
        Utility::Zip => zip::archive(args),
        _ => Err(error!("Utility not implemented"))
    }
}
