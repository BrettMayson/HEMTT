use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::Error;

#[allow(clippy::unnecessary_wraps)] // To match the linux function signature
pub fn windows(arma3: &Path, executable: &str, args: &[String]) -> Result<Command, Error> {
    let mut path = arma3.to_path_buf();
    let exe = PathBuf::from(executable);
    if exe.is_absolute() {
        path = exe;
    } else {
        path.push(exe);
    }
    path.set_extension("exe");
    info!(
        "Launching {:?} with:\n  {}",
        arma3.display(),
        args.join("\n  ")
    );
    let mut cmd = std::process::Command::new(path);
    cmd.args(args);
    Ok(cmd)
}

pub fn linux(args: &[String]) -> Result<Command, Error> {
    // check if flatpak steam is installed
    let flatpak = std::process::Command::new("flatpak")
        .arg("list")
        .arg("--app")
        .output()
        .map_or_else(
            |_| false,
            |o| String::from_utf8_lossy(&o.stdout).contains("com.valvesoftware.Steam"),
        );
    let cmd = if flatpak {
        warn!(
            "A flatpak override will be created to grant Steam access to the .hemttout directory"
        );
        info!("Using flatpak steam with:\n  {}", args.join("\n  "));
        std::process::Command::new("flatpak")
            .arg("override")
            .arg("--user")
            .arg("com.valvesoftware.Steam")
            .arg(format!("--filesystem={}", {
                let mut path = std::env::current_dir()?;
                path.push(".hemttout/dev");
                path.display().to_string()
            }))
            .spawn()?
            .wait()?;
        let mut cmd = std::process::Command::new("flatpak");
        cmd.arg("run")
            .arg("com.valvesoftware.Steam")
            .arg("-applaunch")
            .arg("107410")
            .arg("-nolauncher")
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        cmd
    } else if std::path::Path::new("/bin/distrobox-host-exec").exists()
        || std::path::Path::new("/usr/bin/distrobox-host-exec").exists()
    {
        info!(
            "Using distrobox-host-exec steam with:\n  {}",
            args.join("\n  ")
        );
        let mut cmd = std::process::Command::new("distrobox-host-exec");
        cmd.arg("steam")
            .arg("-applaunch")
            .arg("107410")
            .arg("-nolauncher")
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        cmd
    } else {
        info!("Using native steam with:\n  {}", args.join("\n  "));
        let mut cmd = std::process::Command::new("steam");
        cmd.arg("-applaunch")
            .arg("107410")
            .arg("-nolauncher")
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        cmd
    };
    Ok(cmd)
}
