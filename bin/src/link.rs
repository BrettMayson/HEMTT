use std::path::PathBuf;

use crate::error::Error;

/// Create a symbolic link
///
/// # Errors
/// - [`Error::Link`] if the link could not be created
/// - [`std::io::Error`] if the link command could not be executed
///
/// # Panics
/// - If a symlinks exists and points to a real location, but fails to be read
pub fn create_link(link: &PathBuf, target: &PathBuf) -> Result<(), Error> {
    if cfg!(windows) {
        windows(link, target)
    } else {
        unix(link, target)
    }
}

fn windows(link: &PathBuf, target: &PathBuf) -> Result<(), Error> {
    use std::process::Command;

    if link.is_symlink() {
        if link.exists() {
            if &fs_err::read_link(link).expect("link exists") == target {
                return Ok(());
            }
            warn!(
                "link {:?} already exists, intended to point to {:?}",
                link, target
            );
            if dialoguer::Confirm::new()
                .with_prompt(
                    format!(
                        "A link pointing to `{}` already exists. Do you want to replace it with a link pointing to `{}`?",
                        link.display(),
                        target.display()
                    ))
                .interact()?
            {
                trace!("removing symlink {:?}", link);
                let out = Command::new("cmd")
                    .arg("/C")
                    .arg("rmdir")
                    .arg(link)
                    .output()?;
                if !out.status.success() {
                    return Err(Error::Link(
                        String::from_utf8_lossy(&out.stderr).to_string(),
                    ));
                }
            } else {
                return Ok(());
            }
        } else {
            trace!("removing broken symlink {:?}", link);
            let out = Command::new("cmd")
                .arg("/C")
                .arg("rmdir")
                .arg(link)
                .output()?;
            if !out.status.success() {
                return Err(Error::Link(
                    String::from_utf8_lossy(&out.stderr).to_string(),
                ));
            }
        }
    }

    // attempt junction
    trace!("junction link {:?} => {:?}", link, target);
    let mut out = Command::new("cmd")
        .arg("/C")
        .arg("mklink")
        .arg("/J")
        .arg(link)
        .arg(target)
        .output()?;

    if !out.status.success() {
        // fall-back to directory symbolic link
        trace!("directory symbolic link {:?} => {:?}", link, target);
        out = Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/D")
            .arg(link)
            .arg(target)
            .output()?;

        if !out.status.success() {
            return Err(Error::Link(
                String::from_utf8_lossy(&out.stderr).to_string(),
            ));
        }
    }
    Ok(())
}

fn unix(link: &PathBuf, target: &PathBuf) -> Result<(), Error> {
    if link.exists() {
        warn!(
            "link {:?} already exists, intended to point to {:?}",
            link, target
        );
        return Ok(());
    }
    trace!("symlink {:?} => {:?}", link, target);
    std::os::unix::fs::symlink(target, link)?;
    Ok(())
}
