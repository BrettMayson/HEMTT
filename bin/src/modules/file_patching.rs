use std::sync::Arc;

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
        fs_err::create_dir_all(
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
        self.create(ctx)
    }
}

impl FilePatching {
    /// Get the symlink path with validation
    ///
    /// # Errors
    /// Returns a `Report` with warnings if Arma 3 is not found or mainprefix is not set
    fn get_link_path(&self, ctx: &Context) -> Result<std::path::PathBuf, Report> {
        let mut report = Report::new();
        let Some(arma3dir) = &self.arma3dir else {
            report.push(ArmaNotFound::code());
            return Err(report);
        };
        let Some(mainprefix) = ctx.config().mainprefix() else {
            report.push(NoMainPrefix::code());
            return Err(report);
        };
        let prefix_folder = arma3dir.join(mainprefix);
        Ok(prefix_folder.join(ctx.config().prefix()))
    }

    /// Create the symlink in the Arma 3 directory for file patching
    ///
    /// # Errors
    /// [`Error`] if the link cannot be created, or if the Arma 3 directory cannot be found
    ///
    /// # Panics
    /// If the link's parent directory cannot be resolved
    pub fn create(&self, ctx: &Context) -> Result<Report, Error> {
        let link = match self.get_link_path(ctx) {
            Ok(path) => path,
            Err(report) => return Ok(report),
        };

        let prefix_folder = link.parent().expect("link has parent");
        if !prefix_folder.exists() {
            fs_err::create_dir_all(prefix_folder)?;
        }

        if link.exists() {
            trace!("removing existing symlink at {}", link.display());
            #[cfg(windows)]
            fs_err::remove_dir(&link)?;
            #[cfg(not(windows))]
            fs_err::remove_file(&link)?;
        }
        create_link(&link, ctx.project_folder())?;
        info!("Symlink created at {}", link.display());
        Ok(Report::new())
    }

    /// Remove the symlink from the Arma 3 directory
    ///
    /// # Errors
    /// [`Error`] if the link cannot be removed, or if the Arma 3 directory cannot be found
    pub fn remove(&self, ctx: &Context) -> Result<Report, Error> {
        let link = match self.get_link_path(ctx) {
            Ok(path) => path,
            Err(report) => return Ok(report),
        };

        if link.exists() {
            trace!("removing symlink at {}", link.display());
            #[cfg(windows)]
            fs_err::remove_dir(&link)?;
            #[cfg(not(windows))]
            fs_err::remove_file(&link)?;
            info!("Symlink removed from {}", link.display());
        } else {
            info!("No symlink found at {}", link.display());
        }
        Ok(Report::new())
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
