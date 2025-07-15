use std::sync::{Arc, Mutex};

use hemtt_stringtable::{
    Project,
    analyze::{lint_all, lint_check, lint_one},
    rapify::convert_stringtable,
};
use hemtt_workspace::{
    WorkspacePath,
    reporting::{Code, Diagnostic, Severity},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{Error, context::Context, progress::progress_bar, report::Report};

use super::Module;

#[derive(Debug, Default)]
pub struct Stringtables;
impl Stringtables {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Module for Stringtables {
    fn name(&self) -> &'static str {
        "Stringtables"
    }
    fn priority(&self) -> i32 {
        4000
    }

    fn check(&self, ctx: &crate::context::Context) -> Result<crate::report::Report, crate::Error> {
        let mut report = Report::new();
        report.extend(lint_check(
            ctx.config().lints().stringtables().clone(),
            ctx.config().runtime().clone(),
        ));
        Ok(report)
    }

    fn pre_build(&self, ctx: &Context) -> Result<Report, Error> {
        let report = Arc::new(Mutex::new(Report::new()));
        let mut paths = Vec::new();
        for root in ["addons", "optionals"] {
            if !ctx.workspace_path().join(root)?.exists()? {
                continue;
            }
            paths.extend(
                ctx.workspace_path()
                    .join(root)
                    .expect("vfs issue")
                    .walk_dir()
                    .expect("vfs issue")
                    .into_iter()
                    .filter(|p| {
                        let lower = p.filename().to_lowercase();
                        if lower == "stringtable.csv" || lower == "stringtable.bin" {
                            warn!("Stringtable [{}] will not be linted", p.as_str());
                        }
                        lower == "stringtable.xml"
                    }),
            );
        }
        let length = paths.len();
        let progress = progress_bar(length as u64).with_message("Processing Stringtables");
        let results = paths
            .into_par_iter()
            .map(|path| match Project::read(path.clone()) {
                Ok(project) => {
                    let codes = lint_one(&project, Some(ctx.config()), ctx.addons().to_vec());
                    if !codes.iter().any(|c| c.severity() == Severity::Error) {
                        convert_stringtable(&project);
                    }
                    progress.inc(1);
                    Some((project, codes))
                }
                Err(e) => {
                    debug!("Failed to parse stringtable for {}: {}", path, e);
                    report
                        .lock()
                        .expect("can lock")
                        .push(Arc::new(CodeStringtableInvalid::new(path, e.to_string())));
                    None
                }
            })
            .filter_map(|x| x)
            .collect::<Vec<_>>();

        let mut report = Arc::into_inner(report)
            .expect("last reference")
            .into_inner()
            .expect("can lock");
        let mut stringtables = Vec::new();
        for (project, codes) in results {
            report.extend(codes);
            stringtables.push(project);
        }
        progress.set_message("Linting Stringtables");
        report.extend(lint_all(
            &stringtables,
            Some(ctx.config()),
            ctx.addons().to_vec(),
        ));
        progress.finish_and_clear();
        info!("Checked {} stringtables", length);
        Ok(report)
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeStringtableInvalid {
    path: WorkspacePath,
    reason: String,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeStringtableInvalid {
    fn ident(&self) -> &'static str {
        "INVALID-STRINGTABLE"
    }

    fn message(&self) -> String {
        format!("Stringtable at `{}` is invalid", self.path)
    }

    fn note(&self) -> Option<String> {
        Some(self.reason.clone())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeStringtableInvalid {
    #[must_use]
    pub fn new(path: WorkspacePath, reason: String) -> Self {
        Self {
            path,
            reason,
            diagnostic: None,
        }
        .generate_processed()
    }

    fn generate_processed(mut self) -> Self {
        self.diagnostic = Some(Diagnostic::from_code(&self));
        self
    }
}
