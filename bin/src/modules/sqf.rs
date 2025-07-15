use std::sync::Arc;

use hemtt_common::version::Version;
use hemtt_preprocessor::Processor;
use hemtt_sqf::{
    analyze::{analyze, lint_all, lint_check},
    parser::{ParserError, database::Database},
};
use hemtt_workspace::reporting::{Code, CodesExt, Diagnostic, Severity};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{context::Context, error::Error, progress::progress_bar, report::Report};

use super::Module;

#[derive(Default)]
pub struct SQFCompiler {
    pub database: Option<Arc<Database>>,
}

impl SQFCompiler {
    #[must_use]
    pub const fn new() -> Self {
        Self { database: None }
    }
}

impl Module for SQFCompiler {
    fn name(&self) -> &'static str {
        "SQF"
    }
    fn priority(&self) -> i32 {
        3000
    }

    fn init(&mut self, ctx: &Context) -> Result<Report, Error> {
        self.database = Some(Arc::new(Database::a3_with_workspace(
            ctx.workspace_path(),
            false,
        )?));
        Ok(Report::new())
    }

    fn check(&self, ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();
        report.extend(lint_check(
            ctx.config().lints().sqf().clone(),
            ctx.config().runtime().clone(),
        ));
        Ok(report)
    }

    #[allow(clippy::too_many_lines)]
    fn pre_build(&self, ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();
        let sqf_ext = Some(String::from("sqf"));
        let mut entries = Vec::new();
        for addon in ctx.addons() {
            let addon = Arc::new(addon.clone());
            for entry in ctx.workspace_path().join(addon.folder())?.walk_dir()? {
                if entry.is_file()? {
                    if entry.extension() != sqf_ext || entry.filename().ends_with(".inc.sqf") {
                        continue;
                    }
                    entries.push((addon.clone(), entry));
                }
            }
        }
        let database = self
            .database
            .as_ref()
            .expect("database not initialized")
            .clone();
        let progress = progress_bar(entries.len() as u64).with_message("Compiling SQF");
        let reports = entries
            .par_iter()
            .map(|(addon, entry)| {
                trace!("sqf compiling {}", entry);
                let mut report = Report::new();
                let processed = match Processor::run(entry).map_err(|(_, e)| e) {
                    Ok(p) => p,
                    Err(e) => {
                        if let hemtt_preprocessor::Error::Code(code) = e {
                            report.push(code);
                            return Ok(report);
                        }
                        return Err(e.into());
                    }
                };
                for warning in processed.warnings() {
                    report.push(warning.clone());
                }
                match hemtt_sqf::parser::run(&database, &processed) {
                    Ok(sqf) => {
                        let (codes, sqf_report) = analyze(
                            &sqf,
                            Some(ctx.config()),
                            &processed,
                            addon.clone(),
                            database.clone(),
                        );
                        if let Some(sqf_report) = sqf_report {
                            sqf_report.push_to_addon(addon);
                        }
                        if !codes.failed() {
                            let mut out = entry.with_extension("sqfc")?.create_file()?;
                            sqf.optimize().compile_to_writer(&processed, &mut out)?;
                            progress.inc(1);
                        }
                        for code in codes {
                            report.push(code);
                        }
                        Ok(report)
                    }
                    Err(ParserError::ParsingError(e)) => {
                        if processed.as_str().starts_with("force ")
                            || processed.as_str().contains("\nforce ")
                        {
                            warn!("skipping apparent CBA settings file: {}", entry);
                        } else {
                            for error in e {
                                report.push(error);
                            }
                        }
                        Ok(report)
                    }
                    Err(ParserError::LexingError(e)) => {
                        for error in e {
                            report.push(error);
                        }
                        Ok(report)
                    }
                }
            })
            .collect::<Result<Vec<Report>, Error>>()?;
        for new_report in reports {
            report.merge(new_report);
        }
        progress.finish_and_clear();
        info!("Compiled {} sqf files", entries.len());

        report.extend(lint_all(
            Some(ctx.config()),
            &ctx.addons().to_vec(),
            database,
        ));

        Ok(report)
    }
    fn post_build(&self, ctx: &Context) -> Result<Report, crate::Error> {
        let mut report = Report::new();
        let mut required_version = Version::new(0, 0, 0, None);
        let mut required_by = Vec::new();
        for addon in ctx.addons() {
            let addon_version = addon.build_data().required_version();
            if let Some((version, _, _)) = addon_version {
                if version > required_version {
                    required_version = version;
                    required_by = vec![addon.name().to_string()];
                } else if version == required_version {
                    required_by.push(addon.name().to_string());
                }
            }
        }

        let database = self.database.as_ref().expect("database not initialized");

        let wiki_version = arma3_wiki::model::Version::new(
            u8::try_from(required_version.major()).unwrap_or_default(),
            u8::try_from(required_version.minor()).unwrap_or_default(),
        );
        if database.wiki().version() < &wiki_version {
            report.push(Arc::new(RequiresFutureVersion::new(
                wiki_version,
                required_by,
                *database.wiki().version(),
            )));
        }

        Ok(report)
    }
}

pub struct RequiresFutureVersion {
    required_version: arma3_wiki::model::Version,
    required_by: Vec<String>,
    wiki_version: arma3_wiki::model::Version,
}
impl Code for RequiresFutureVersion {
    fn ident(&self) -> &'static str {
        "BSW1"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn message(&self) -> String {
        format!(
            "Required version `{}` is higher than the current stable `{}`",
            self.required_version, self.wiki_version
        )
    }

    fn note(&self) -> Option<String> {
        Some(format!(
            "addons requiring version `{}`: {}",
            self.required_version,
            self.required_by.join(", ")
        ))
    }

    fn help(&self) -> Option<String> {
        Some("Learn about the `development` branch at `https://community.bistudio.com/wiki/Arma_3:_Steam_Branches`".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl RequiresFutureVersion {
    pub const fn new(
        required_version: arma3_wiki::model::Version,
        required_by: Vec<String>,
        wiki_version: arma3_wiki::model::Version,
    ) -> Self {
        Self {
            required_version,
            required_by,
            wiki_version,
        }
    }
}
