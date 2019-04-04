use serde::Deserialize;

pub mod convert;
pub mod translation;
pub mod zip;

#[derive(Debug, Deserialize)]
pub enum Utility {
    ConvertProject,
    Template,
    Translation,
    Zip,
}

pub fn find(utility: &str) -> Option<Utility> {
    return match utility {
        "convertproject" => Some(Utility::ConvertProject),
        "template" => Some(Utility::Template),
        "translation" => Some(Utility::Translation),
        "zip" => Some(Utility::Zip),
        _ => None
    }
}

pub fn run(utility: &Utility, args: &mut Vec<String>) -> Result<(), std::io::Error> {
    #[allow(unreachable_patterns)]
    return match utility {
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
