use hemtt_format::{CONFIG_EXTENSIONS, FormatterConfig, SQF_EXTENSIONS};

#[derive(clap::Parser)]
/// Format Config and SQF files
pub struct Command {}

const IGNORED_FOLDERES: &[&str] = &[".hemttout", ".git", "include/"];

/// Execute the format command
///
/// # Errors
/// [`Error`] depending on the modules
///
/// # Panics
/// If a name is not provided, but this is usually handled by clap
pub fn execute(_cmd: &Command) -> ! {
    let mut count = 0;
    let mut errors = 0;
    for entry in walkdir::WalkDir::new(".") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        let path_string = path.display().to_string();
        if IGNORED_FOLDERES
            .iter()
            .any(|&folder| path_string.contains(folder))
        {
            continue;
        }
        if path.is_file() {
            let ext = path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            if CONFIG_EXTENSIONS.contains(&ext) {
                let content = std::fs::read_to_string(path)
                    .unwrap_or_else(|_| panic!("Failed to read file {}", path.display()));
                match hemtt_format::format_config(&content, &FormatterConfig::default()) {
                    Ok(formatted) => {
                        if formatted != content {
                            std::fs::write(path, formatted).unwrap_or_else(|_| {
                                panic!("Failed to write file {}", path.display())
                            });
                            debug!("Formatted {}", path.display());
                            count += 1;
                        }
                    }
                    Err(err) => {
                        error!("Failed to format {}: {}", path.display(), err);
                        errors += 1;
                    }
                }
            }
            if SQF_EXTENSIONS.contains(&ext) {
                let content = std::fs::read_to_string(path)
                    .unwrap_or_else(|_| panic!("Failed to read file {}", path.display()));
                match hemtt_format::format_sqf(&content, &FormatterConfig::default()) {
                    Ok(formatted) => {
                        if formatted != content {
                            std::fs::write(path, formatted).unwrap_or_else(|_| {
                                panic!("Failed to write file {}", path.display())
                            });
                            debug!("Formatted {}", path.display());
                            count += 1;
                        }
                    }
                    Err(err) => {
                        error!("Failed to format {}: {}", path.display(), err);
                        errors += 1;
                    }
                }
            }
        }
    }
    info!("Formatted {} files", count);
    if errors > 0 {
        error!("Encountered {} errors during formatting", errors);
    }
    std::process::exit(i32::from(errors > 0))
}
