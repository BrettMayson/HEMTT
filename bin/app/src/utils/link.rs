use hemtt_bin_error::Error;

#[allow(clippy::module_name_repetitions)]
#[cfg(windows)]
pub fn create_link(link: &str, target: &str) -> Result<(), Error> {
    use std::{process::Command, time::Duration};
    trace!("link {:?} => {:?}", link, target);
    std::thread::sleep(Duration::from_millis(100));
    let out = Command::new("cmd")
        .arg("/C")
        .arg("mklink")
        .arg("/J")
        .arg(link)
        .arg(target)
        .output()?;
    if !out.status.success() {
        return Err(Error::Link(
            String::from_utf8_lossy(&out.stderr).to_string(),
        ));
    }
    Ok(())
}

#[allow(clippy::module_name_repetitions)]
#[cfg(not(windows))]
pub fn create_link(link: &str, target: &str) -> Result<(), Error> {
    let target = target.replace('\\', "/");
    trace!("link {:?} => {:?}", link, target);
    std::os::unix::fs::symlink(target, link)?;
    Ok(())
}
