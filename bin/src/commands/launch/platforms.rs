use std::{
    path::{Path, PathBuf},
    process::Child,
};

use crate::Error;

pub fn windows(arma3: &Path, executable: &str, args: &[String]) -> Result<Child, Error> {
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
    Ok(std::process::Command::new(path).args(args).spawn()?)
}

pub fn linux(args: &[String]) -> Result<Child, Error> {
    // check if flatpak steam is installed
    let flatpak = std::process::Command::new("flatpak")
        .arg("list")
        .arg("--app")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("com.valvesoftware.Steam"))?;
    let child = if flatpak {
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
        std::process::Command::new("flatpak")
            .arg("run")
            .arg("com.valvesoftware.Steam")
            .arg("-applaunch")
            .arg("107410")
            .arg("-nolauncher")
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?
    } else if std::path::Path::new("/bin/distrobox-host-exec").exists()
        || std::path::Path::new("/usr/bin/distrobox-host-exec").exists()
    {
        info!(
            "Using distrobox-host-exec steam with:\n  {}",
            args.join("\n  ")
        );
        std::process::Command::new("distrobox-host-exec")
            .arg("steam")
            .arg("-applaunch")
            .arg("107410")
            .arg("-nolauncher")
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?
    } else {
        info!("Using native steam with:\n  {}", args.join("\n  "));
        std::process::Command::new("steam")
            .arg("-applaunch")
            .arg("107410")
            .arg("-nolauncher")
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?
    };
    Ok(child)
}
