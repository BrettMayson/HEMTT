use std::io::Write;

use hemtt::{HEMTTError, Project};

pub fn run(p: &mut Project, a: &clap::ArgMatches) -> Result<(), HEMTTError> {
    match a.subcommand() {
        ("inc", Some(sa)) => {
            match sa.subcommand() {
                ("major", _) => p.version_mut().increment_major(),
                ("minor", _) => p.version_mut().increment_minor(),
                ("patch", _) => p.version_mut().increment_patch(),
                _ => {
                    return Err(HEMTTError::User(String::from(
                        "options are `major`, `minor`, `patch`",
                    )))
                }
            }
            let mut out = create_file!("./.hemtt/base.toml")?;
            out.write_fmt(format_args!(
                "{}",
                toml::to_string(&p).map_err(|e| HEMTTError::Generic(e.to_string()))?
            ))?;
            info!("Version: {}", p.version());
        }
        ("", None) => {}
        _ => unimplemented!(),
    }
    Ok(())
}
