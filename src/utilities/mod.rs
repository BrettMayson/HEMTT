use serde::Deserialize;

pub mod translation;
pub mod convert;

#[derive(Debug, Deserialize)]
pub enum Utility {
    Convert,
    Translation
}

pub fn find(utility: &str) -> Option<Utility> {
    return match utility {
        "convert" => Some(Utility::Convert),
        "translation" => Some(Utility::Translation),
        _ => None
    }
}

pub fn run(utility: &Utility) -> Result<(), std::io::Error> {
    #[allow(unreachable_patterns)]
    return match utility {
        Utility::Convert => convert::run(),
        Utility::Translation => translation::check(),
        _ => Err(error!("Utility not implemented"))
    }
}
