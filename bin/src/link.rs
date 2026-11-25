use std::path::PathBuf;

use crate::error::Error;

#[allow(clippy::module_name_repetitions)]
#[cfg(windows)]
/// Create a symbolic link
///
/// # Errors
/// - [`Error::Link`] if the link could not be created
/// - [`std::io::Error`] if the link command could not be executed
///
/// # Panics
/// - If a symlinks exists and points to a real location, but fails to be read
pub fn create_link(link: &PathBuf, target: &PathBuf) -> Result<(), Error> {
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

#[allow(clippy::module_name_repetitions)]
#[cfg(not(windows))]
/// Create a symbolic link
///
/// # Errors
/// - [`std::io::Error`] if the link could not be created
pub fn create_link(link: &PathBuf, target: &PathBuf) -> Result<(), Error> {
    if link.is_symlink() && !link.exists() {
        // `exists` follows the link, a broken link would otherwise fail with "File exists"
        trace!("removing broken symlink {:?}", link);
        fs_err::remove_file(link)?;
    }
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

#[must_use]
/// Does anything exist at `path`, including a broken symlink
pub fn link_or_file_exists(path: &std::path::Path) -> bool {
    fs_err::symlink_metadata(path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::{create_link, link_or_file_exists};

    #[test]
    fn broken_link_exists() {
        let _directory = hemtt_test::directory::TemporaryDirectory::new();
        let root = std::env::current_dir().expect("current dir");
        let target = root.join("target");
        let link = root.join("link");
        assert!(!link_or_file_exists(&link));
        fs_err::create_dir(&target).expect("create target");
        create_link(&link, &target).expect("create link");
        fs_err::remove_dir(&target).expect("remove target");
        assert!(!link.exists(), "link is broken");
        assert!(link_or_file_exists(&link));
    }

    #[test]
    fn replaces_broken_link() {
        let _directory = hemtt_test::directory::TemporaryDirectory::new();
        let root = std::env::current_dir().expect("current dir");
        let old_target = root.join("old");
        let new_target = root.join("new");
        let link = root.join("link");
        fs_err::create_dir(&old_target).expect("create old target");
        fs_err::create_dir(&new_target).expect("create new target");
        create_link(&link, &old_target).expect("create link");
        // the old build folder is removed, e.g. by `hemtt clean`
        fs_err::remove_dir(&old_target).expect("remove old target");

        create_link(&link, &new_target).expect("replace broken link");
        assert!(link.exists());
        assert_eq!(
            fs_err::canonicalize(&link).expect("resolve link"),
            fs_err::canonicalize(&new_target).expect("resolve target")
        );
    }
}
