use serde::Deserialize;

pub mod translation;
pub mod convert;

#[derive(Debug, Deserialize)]
pub enum Utility {
    ConvertProject,
    Translation
}

pub fn find(utility: &str) -> Option<Utility> {
    return match utility {
        "convertproject" => Some(Utility::ConvertProject),
        "translation" => Some(Utility::Translation),
        _ => None
    }
}

pub fn run(utility: &Utility) -> Result<(), std::io::Error> {
    #[allow(unreachable_patterns)]
    return match utility {
        Utility::ConvertProject => convert::run(),
        Utility::Translation => translation::check(),
        _ => Err(error!("Utility not implemented"))
    }
}
