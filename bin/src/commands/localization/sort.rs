use std::{io::BufReader, path::PathBuf};

use hemtt_stringtable::Project;

use crate::{context::Context, report::Report, Error};

#[derive(clap::Parser)]
#[allow(clippy::module_name_repetitions)]
#[command(verbatim_doc_comment)]
/// Sorts the stringtables
///
/// HEMTT will:
///
/// 1. Sort the Packages in alphabetical order.
/// 2. Sort the Containers in alphabetical order (if any).
/// 3. Sort the Keys in alphabetical order.
/// 4. Sort the Localized Strings in the order of [this table](https://community.bistudio.com/wiki/Stringtable.xml#Supported_Languages)
pub struct Command {
    #[arg(long)]
    /// Only sort the languages within keys
    only_lang: bool,
}

/// Sort the stringtables
///
/// # Errors
/// [`Error`] depending on the modules
pub fn sort(cmd: &Command) -> Result<Report, Error> {
    let ctx = Context::new(None, crate::context::PreservePrevious::Remove, true)?;

    for root in ["addons", "optionals"] {
        if !ctx.project_folder().join(root).exists() {
            continue;
        }
        let paths: Vec<PathBuf> = walkdir::WalkDir::new(ctx.project_folder().join(root))
            .into_iter()
            .filter_map(|p| {
                p.map(|p| {
                    if p.file_name() == "stringtable.xml" {
                        Some(p.path().to_path_buf())
                    } else {
                        None
                    }
                })
                .ok()
                .flatten()
            })
            .collect::<Vec<_>>();
        for path in paths {
            if path.exists() {
                let mut file = std::fs::File::open(&path)?;
                match Project::from_reader(BufReader::new(&mut file)) {
                    Ok(mut project) => {
                        if !cmd.only_lang {
                            project.sort();
                        }
                        let mut writer = String::new();
                        if let Err(e) = project.to_writer(&mut writer) {
                            error!("Failed to write stringtable for {}", path.display());
                            error!("{:?}", e);
                            return Ok(Report::new());
                        }
                        if let Err(e) = std::fs::write(&path, writer) {
                            error!("Failed to write stringtable for {}", path.display());
                            error!("{:?}", e);
                            return Ok(Report::new());
                        }
                    }
                    Err(e) => {
                        error!("Failed to read stringtable for {}", path.display());
                        error!("{:?}", e);
                        return Ok(Report::new());
                    }
                };
            }
        }
    }
    Ok(Report::new())
}
