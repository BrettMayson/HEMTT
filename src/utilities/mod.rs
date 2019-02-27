use serde::Deserialize;

pub mod translation;

#[derive(Debug, Deserialize)]
pub enum Utility {
    Translation
}

pub fn find(utility: &str) -> Option<Utility> {
    return match utility {
        "translation" => Some(Utility::Translation),
        _ => None
    }
}

pub fn run(utility: &Utility) -> Result<(), std::io::Error> {
    match utility {
        Utility::Translation => {
            return translation::check();
        }
    }
    #[allow(unreachable_code)]
    Err(error!("Utility not implemented"))
}
