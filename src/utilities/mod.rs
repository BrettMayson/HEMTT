use serde::Deserialize;

use std::io::{Error};

pub mod translation;
pub mod zip;

#[derive(Debug, Deserialize)]
pub enum Utility {
    Translation,
    Zip
}

pub fn find(utility: &str) -> Option<Utility> {
    return match utility {
        "translation" => Some(Utility::Translation),
        "zip" => Some(Utility::Zip),
        _ => None
    }
}

pub fn run(utility: &Utility) -> Result<(), Error> {
    match utility {
        Utility::Translation => {
            return translation::check();
        }
        Utility::Zip => {
            return zip::archive();
        }
    }
    #[allow(unreachable_code)]
    Err(error!("Utility not implemented"))
}
