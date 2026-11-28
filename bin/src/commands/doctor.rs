use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::{
    pdrive::PDrive,
    reporting::{Code, Diagnostic},
};

use crate::{Error, report::Report};

#[derive(clap::Parser)]
/// Check your environment for common problems
///
/// Reports what HEMTT can find on this machine, and anything that will stop
/// a command from working:
///
/// - The HEMTT version, and whether an update is available
/// - Arma 3 and Arma 3 Tools
/// - Wine or Proton, on Linux, for binarizing
/// - The P drive
/// - The project configuration, when run in a project
/// - Broken links in the Arma 3 folder
///
/// Include the output when asking for help.
pub struct Command {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok,
    Info,
    Warn,
    Error,
}

impl Status {
    const fn symbol(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Info => "--",
            Self::Warn => "!!",
            Self::Error => "XX",
        }
    }

    const fn color(self) -> &'static str {
        match self {
            Self::Ok => "\x1b[32m",
            Self::Info => "\x1b[36m",
            Self::Warn => "\x1b[33m",
            Self::Error => "\x1b[31m",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Check {
    pub name: &'static str,
    pub status: Status,
    pub detail: String,
    pub help: Option<String>,
}

impl Check {
    fn new(name: &'static str, status: Status, detail: impl Into<String>) -> Self {
        Self {
            name,
            status,
            detail: detail.into(),
            help: None,
        }
    }

    fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    fn render(&self, color: bool) -> String {
        let symbol = if color {
            format!("{}[{}]\x1b[0m", self.status.color(), self.status.symbol())
        } else {
            format!("[{}]", self.status.symbol())
        };
        let mut out = format!("{symbol} {:<14} {}", self.name, self.detail);
        if let Some(help) = &self.help {
            for line in help.lines() {
                out.push_str("\n                    ");
                out.push_str(line);
            }
        }
        out
    }
}

/// Execute the doctor command
///
/// # Errors
/// Will not return an error
pub fn execute(_: &Command) -> Result<Report, Error> {
    let checks = run();
    let color = hemtt_common::ansi::ansi_supported();
    for check in &checks {
        println!("{}", check.render(color));
    }
    let errors = checks.iter().filter(|c| c.status == Status::Error).count();
    let warnings = checks.iter().filter(|c| c.status == Status::Warn).count();
    println!();
    println!("{errors} error(s), {warnings} warning(s)");
    let mut report = Report::new();
    if errors > 0 {
        report.push(Arc::new(DoctorFailed(errors)));
    }
    Ok(report)
}

struct DoctorFailed(usize);

impl Code for DoctorFailed {
    fn ident(&self) -> &'static str {
        // binary, command, doctor, error 1
        "BCDE1"
    }

    fn message(&self) -> String {
        format!("hemtt doctor found {} error(s)", self.0)
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

#[must_use]
/// Run every check
pub fn run() -> Vec<Check> {
    let mut checks = vec![version(), platform()];

    let arma3 = hemtt_common::steam::find_arma3();
    checks.push(arma3.as_ref().map_or_else(
        || {
            Check::new("Arma 3", Status::Warn, "not found")
                .help("Needed by `hemtt launch` and file patching.\nInstall Arma 3 with Steam and run it once.")
        },
        |path| Check::new("Arma 3", Status::Ok, path.display().to_string()),
    ));

    checks.push(tools());
    if let Some(check) = compatibility() {
        checks.push(check);
    }
    checks.push(pdrive());

    let project = std::env::current_dir()
        .ok()
        .filter(|dir| dir.join(".hemtt").is_dir());
    match project {
        None => checks.push(Check::new(
            "Project",
            Status::Info,
            "not in a HEMTT project, skipping project checks",
        )),
        Some(root) => checks.extend(project_checks(&root)),
    }

    if let Some(arma3) = &arma3 {
        checks.push(broken_links(arma3));
    }

    checks
}

fn version() -> Check {
    let current = env!("HEMTT_VERSION");
    match crate::update::check() {
        Ok(Some(latest)) => Check::new(
            "HEMTT",
            Status::Warn,
            format!("{current}, {latest} is available"),
        )
        .help("Update HEMTT: https://hemtt.dev/installation/index.html"),
        Ok(None) => Check::new("HEMTT", Status::Ok, current),
        Err(e) => Check::new(
            "HEMTT",
            Status::Info,
            format!("{current}, could not check for updates: {e}"),
        ),
    }
}

fn platform() -> Check {
    let mut detail = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    if crate::is_ci() {
        detail.push_str(", CI");
    }
    if cfg!(target_os = "macos") {
        return Check::new("Platform", Status::Warn, detail)
            .help("Binarizing is not supported on macOS, files are packed without binarizing.");
    }
    Check::new("Platform", Status::Ok, detail)
}

fn tools() -> Check {
    if cfg!(target_os = "macos") {
        return Check::new("Arma 3 Tools", Status::Info, "not supported on macOS");
    }
    crate::modules::locate_binarize().map_or_else(
        || {
            let help = if cfg!(windows) {
                "Install Arma 3 Tools with Steam and run it once, or set HEMTT_BINARIZE_PATH.\nWithout it, files are packed without binarizing."
            } else {
                "Install Arma 3 Tools with Steam, or set HEMTT_BI_TOOLS.\nWithout it, files are packed without binarizing."
            };
            Check::new("Arma 3 Tools", Status::Warn, "Binarize not found").help(help)
        },
        |path| Check::new("Arma 3 Tools", Status::Ok, path.display().to_string()),
    )
}

fn compatibility() -> Option<Check> {
    if cfg!(windows) || cfg!(target_os = "macos") {
        return None;
    }
    Some(crate::modules::CompatibiltyTool::determine().map_or_else(
        || {
            Check::new(
                "Wine",
                Status::Warn,
                "wine64, wine and Proton Sniper not found",
            )
            .help("Binarize runs through Wine or Proton on Linux.")
        },
        |tool| Check::new("Wine", Status::Ok, tool.to_string()),
    ))
}

fn pdrive() -> Check {
    match hemtt_workspace::pdrive::search() {
        Some(PDrive::Tools(_, path)) => {
            Check::new("P drive", Status::Ok, path.display().to_string())
        }
        Some(PDrive::OnDemand(_)) => Check::new(
            "P drive",
            Status::Info,
            "not found, files are read from Arma 3 when needed",
        ),
        None => Check::new("P drive", Status::Info, "not found")
            .help("Only needed by projects with `pdrive = \"require\"`."),
    }
}

fn project_checks(root: &Path) -> Vec<Check> {
    let path = root.join(".hemtt").join("project.toml");
    if !path.exists() {
        return vec![
            Check::new("Project", Status::Error, ".hemtt/project.toml not found")
                .help("Run `hemtt new` to create a project, or create the file."),
        ];
    }
    let config = match ProjectConfig::from_file(&path) {
        Ok(config) => config,
        Err(e) => {
            return vec![Check::new(
                "Project",
                Status::Error,
                format!("invalid .hemtt/project.toml: {e}"),
            )];
        }
    };

    vec![Check::new(
        "Project",
        Status::Ok,
        format!("{} ({})", config.name(), config.prefix()),
    )]
}

/// Links left behind by a project that was moved or deleted
fn broken_links(arma3: &Path) -> Check {
    // file patching links are at `<arma>/<mainprefix>/<prefix>`, a shallow walk finds them
    let broken = find_broken_links(arma3, 3);
    if broken.is_empty() {
        return Check::new("Links", Status::Ok, "no broken links in the Arma 3 folder");
    }
    let mut help = broken
        .iter()
        .take(5)
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>();
    if broken.len() > 5 {
        help.push(format!("and {} more", broken.len() - 5));
    }
    help.push("`hemtt utils remove-links` removes every link in the Arma 3 folder.".to_string());
    Check::new(
        "Links",
        Status::Warn,
        format!("{} broken link(s) in the Arma 3 folder", broken.len()),
    )
    .help(help.join("\n"))
}

fn find_broken_links(root: &Path, depth: usize) -> Vec<PathBuf> {
    walkdir::WalkDir::new(root)
        .max_depth(depth)
        .into_iter()
        .filter_entry(|e| e.file_name() != "!Workshop")
        .filter_map(Result::ok)
        .filter(|e| e.path_is_symlink() && !e.path().exists())
        .map(walkdir::DirEntry::into_path)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn links() {
        let dir = std::env::temp_dir().join(format!("hemtt-doctor-{}", std::process::id()));
        let _ = fs_err::remove_dir_all(&dir);
        let arma = dir.join("arma");
        let project = dir.join("project");
        fs_err::create_dir_all(arma.join("z")).expect("create arma");
        fs_err::create_dir_all(&project).expect("create project");

        let link = arma.join("z").join("test");
        std::os::unix::fs::symlink(&project, &link).expect("link");
        assert_eq!(find_broken_links(&arma, 3), Vec::<PathBuf>::new());

        fs_err::remove_dir_all(&project).expect("remove project");
        assert_eq!(find_broken_links(&arma, 3), vec![link.clone()]);

        fs_err::create_dir_all(arma.join("!Workshop")).expect("create workshop");
        std::os::unix::fs::symlink(dir.join("missing"), arma.join("!Workshop").join("mod"))
            .expect("workshop link");
        assert_eq!(find_broken_links(&arma, 3), vec![link]);

        let _ = fs_err::remove_dir_all(&dir);
    }

    #[test]
    fn render() {
        let check = Check::new("Arma 3", Status::Warn, "not found").help("one\ntwo");
        assert_eq!(
            check.render(false),
            "[!!] Arma 3         not found\n                    one\n                    two"
        );
    }
}
