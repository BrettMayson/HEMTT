use std::process::Command;

use crate::error::Error;

#[allow(clippy::module_name_repetitions)]
pub fn create_link(original: &str, target: &str) -> Result<(), Error> {
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
