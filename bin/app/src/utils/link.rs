use crate::error::Error;

#[allow(clippy::module_name_repetitions)]
#[cfg(windows)]
pub fn create_link(original: &str, target: &str) -> Result<(), Error> {
    use std::process::Command;
    let out = Command::new("cmd")
        .arg("/C")
        .arg("mklink")
        .arg("/J")
        .arg(original)
        .arg(target)
        .output()?;
    if !out.status.success() {
        return Err(Error::Link(
            String::from_utf8_lossy(&out.stderr).to_string(),
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn create_link(original: &str, target: &str) -> Result<(), Error> {
    std::os::unix::fs::symlink(target, original)?;
    Ok(())
}
