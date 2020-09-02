use std::fs::File;
use std::io::Write;

use hemtt::{HEMTTError, Project};

pub fn run(p: &mut Project, a: &clap::ArgMatches) -> Result<(), HEMTTError> {
    match a.subcommand() {
        ("inc", Some(sa)) => {
            match sa.subcommand() {
                ("major", _) => p.version.increment_major(),
                ("minor", _) => p.version.increment_minor(),
                ("patch", _) => p.version.increment_patch(),
                _ => {
                    return Err(HEMTTError::User(String::from(
                        "options are `major`, `minor`, `patch`",
                    )))
                }
            }
            let mut out = File::create("./.hemtt/base.toml")?;
            out.write_fmt(format_args!(
                "{}",
                toml::to_string(&p).map_err(|e| HEMTTError::Generic(e.to_string()))?
            ))?;
            info!("Version: {}", p.version);
        }
        ("", None) => {}
        _ => unimplemented!(),
    }
    Ok(())
}
