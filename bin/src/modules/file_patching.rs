use std::{fs::create_dir_all, sync::Arc};

use hemtt_workspace::reporting::{Code, Diagnostic};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{context::Context, error::Error, link::create_link, report::Report};

use super::Module;

pub struct FilePatching {
    arma3dir: Option<std::path::PathBuf>,
}

impl Default for FilePatching {
    fn default() -> Self {
        Self {
            arma3dir: hemtt_common::steam::find_app(107_410),
        }
    }
}

impl Module for FilePatching {
    fn name(&self) -> &'static str {
        "FilePatching"
    }

    fn pre_build(&self, ctx: &Context) -> Result<Report, Error> {
        create_dir_all(
            ctx.build_folder()
                .expect("build folder exists")
                .join("addons"),
        )?;
        ctx.addons()
            .par_iter()
            .map(|addon| {
                create_link(
                    &ctx.build_folder()
                        .expect("build folder exists")
                        .join("addons")
                        .join(addon.name().replace('/', "\\")),
                    &ctx.project_folder().join(addon.folder_pathbuf()),
                )
            })
            .collect::<Result<(), Error>>()?;
        Ok(Report::new())
    }

    fn post_build(&self, ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();
        let Some(arma3dir) = &self.arma3dir else {
            report.push(ArmaNotFound::code());
            return Ok(report);
        };
        let Some(mainprefix) = ctx.config().mainprefix() else {
            report.push(NoMainPrefix::code());
            return Ok(report);
        };
        let prefix_folder = arma3dir.join(mainprefix);
        if !prefix_folder.exists() {
            std::fs::create_dir_all(&prefix_folder)?;
        }

        let link = prefix_folder.join(ctx.config().prefix());
        if link.exists() {
            trace!("removing existing symlink at {}", link.display());
            std::fs::remove_file(&link)?;
        }
        create_link(&link, ctx.build_folder().expect("build folder exists"))?;
        info!("created symlink to build folder for file patching");
        Ok(report)
    }
}

struct ArmaNotFound;

impl Code for ArmaNotFound {
    fn ident(&self) -> &'static str {
        // binary, module, file patching, warning 1
        "BMFW1"
    }

    fn severity(&self) -> hemtt_workspace::reporting::Severity {
        hemtt_workspace::reporting::Severity::Warning
    }

    fn message(&self) -> String {
        "Arma 3 not found, skipping symlink creation.".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("Install Arma 3 via Steam, and run it at least once.".to_owned())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl ArmaNotFound {
    #[must_use]
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self {})
    }
}

struct NoMainPrefix;

impl Code for NoMainPrefix {
    fn ident(&self) -> &'static str {
        // binary, module, file patching, warning 2
        "BMFW2"
    }

    fn severity(&self) -> hemtt_workspace::reporting::Severity {
        hemtt_workspace::reporting::Severity::Warning
    }

    fn message(&self) -> String {
        "No mainprefix set, skipping symlink creation.".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("Set the mainprefix in the project config.".to_owned())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl NoMainPrefix {
    #[must_use]
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self {})
    }
}
