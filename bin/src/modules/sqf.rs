use std::{io::Write, sync::Arc};

use hemtt_common::version::Version;
use hemtt_sqf::{
    analyze::{create_lint_manager, lint_all, lint_check},
    parser::database::Database,
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

    fn pre_build(&self, ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();
        let mut entries = Vec::new();
        for addon in ctx.addons() {
            let addon = Arc::new(addon.clone());
            let patterns = SqfcPatterns::new(&addon)?;
            for entry in ctx.workspace_path().join(addon.folder())?.walk_dir()? {
                if entry.is_file()? {
                    if !hemtt_sqf::is_compilation_unit(&entry) {
                        continue;
                    }
                    let mode = patterns.mode(
                        entry
                            .as_str()
                            .trim_start_matches(&format!("/{}/", addon.folder())),
                    );
                    entries.push((addon.clone(), entry, mode));
                }
            }
        }
        let database = self
            .database
            .as_ref()
            .expect("database not initialized")
            .clone();
        let manager = match create_lint_manager(Some(ctx.config())) {
            Ok(manager) => Arc::new(manager),
            Err(errors) => {
                report.extend(errors);
                return Ok(report);
            }
        };
        let progress = progress_bar(entries.len() as u64).with_message("Compiling SQF");
        let reports = entries
            .par_iter()
            .map(|(addon, entry, mode)| {
                trace!("sqf compiling {}", entry);
                let mut report = Report::new();
                let processed = match hemtt_preprocessor::Processor::run_with_sources(
                    entry,
                    ctx.config().preprocessor(),
                    ctx.sources(),
                )
                .map_err(|(_, e)| e)
                {
                    Ok(p) => p,
                    Err(e) => {
                        if let hemtt_preprocessor::Error::Code(code) = e {
                            report.push(code);
                            return Ok(report);
                        }
                        return Err(e.into());
                    }
                };
                let checked = hemtt_sqf::check::check(
                    &processed,
                    Some(ctx.config()),
                    addon,
                    entry,
                    database.clone(),
                    Some(manager.clone()),
                );
                if let Some(sqf_report) = checked.report {
                    sqf_report.push_to_addon(addon);
                }
                if let Some(sqf) = checked.statements
                    && !checked.codes.failed()
                {
                    if *mode == SqfcMode::Exclude {
                        trace!("sqfc excluded for {}", entry);
                    } else if processed.no_rapify() {
                        // e.g. `#pragma hemtt flag pe26_ignore_unsupported_builtin`,
                        // the game has to preprocess the plain SQF itself
                        trace!(
                            "sqfc skipped for {}, it must be preprocessed by the game",
                            entry
                        );
                    } else {
                        // The variant the game uses when started with `-debug`
                        let debug = if processed.uses_debug_macro() {
                            debug_variant(ctx, entry, &database, &processed, &mut report)?
                                .map(|(sqf, processed)| (sqf.optimize(), processed))
                        } else {
                            None
                        };
                        let mut buffer = Vec::new();
                        match (
                            sqf.optimize().compile_with_debug_to_writer(
                                &processed,
                                debug.as_ref().map(|(sqf, processed)| (sqf, processed)),
                                &mut buffer,
                            ),
                            mode,
                        ) {
                            (Ok(()), SqfcMode::Compile) => {
                                entry
                                    .with_extension("sqfc")?
                                    .create_file()?
                                    .write_all(&buffer)?;
                            }
                            (Ok(()), _) => {
                                report.push(Arc::new(SqfcExcludeNotNeeded {
                                    file: entry.as_str().to_string(),
                                }));
                            }
                            (Err(e), SqfcMode::Compile) => {
                                report.push(Arc::new(SqfcFailed {
                                    file: entry.as_str().to_string(),
                                    reason: e.to_string(),
                                }));
                            }
                            (Err(e), _) => {
                                debug!("sqfc temporarily excluded for {}: {}", entry, e);
                            }
                        }
                    }
                    progress.inc(1);
                }
                for code in checked.codes {
                    report.push(code);
                }
                Ok(report)
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
            &manager,
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

/// Preprocess and parse `entry` as the game would when started with `-debug`
///
/// Returns `None` when `__A3_DEBUG__` does not change the output,
/// or when the debug variant has errors, which are added to `report`
fn debug_variant(
    ctx: &Context,
    entry: &hemtt_workspace::WorkspacePath,
    database: &Database,
    processed: &hemtt_workspace::reporting::Processed,
    report: &mut Report,
) -> Result<Option<(hemtt_sqf::Statements, hemtt_workspace::reporting::Processed)>, Error> {
    let debug_processed = match hemtt_preprocessor::Processor::run_with_sources(
        entry,
        &ctx.config().preprocessor().clone().with_debug(true),
        ctx.sources(),
    )
    .map_err(|(_, e)| e)
    {
        Ok(processed) => processed,
        Err(hemtt_preprocessor::Error::Code(code)) => {
            report.push(code);
            return Ok(None);
        }
        Err(e) => return Err(e.into()),
    };
    if debug_processed.as_str() == processed.as_str() {
        trace!("`__A3_DEBUG__` does not change {}", entry);
        return Ok(None);
    }
    match hemtt_sqf::parser::run(database, &debug_processed) {
        Ok(statements) => Ok(Some((statements, debug_processed))),
        Err(
            hemtt_sqf::parser::ParserError::ParsingError(errors)
            | hemtt_sqf::parser::ParserError::LexingError(errors),
        ) => {
            for error in errors {
                report.push(error);
            }
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How a file should be handled for SQFC, from `addon.toml` `[sqfc]`
pub enum SqfcMode {
    /// Compile and include the SQFC
    Compile,
    /// `sqfc.exclude`: never compile, the plain SQF is used
    Exclude,
    /// `sqfc.exclude_temporary`: compile to check, but don't include the SQFC
    ExcludeTemporary,
}

/// Compiled `[sqfc]` glob patterns for an addon
struct SqfcPatterns {
    exclude: Vec<glob::Pattern>,
    exclude_temporary: Vec<glob::Pattern>,
}

impl SqfcPatterns {
    fn new(addon: &hemtt_workspace::addons::Addon) -> Result<Self, glob::PatternError> {
        let compile = |patterns: &[String]| {
            patterns
                .iter()
                .map(|p| glob::Pattern::new(p))
                .collect::<Result<Vec<_>, _>>()
        };
        let Some(config) = addon.config() else {
            return Ok(Self {
                exclude: Vec::new(),
                exclude_temporary: Vec::new(),
            });
        };
        Ok(Self {
            exclude: compile(config.sqfc().exclude())?,
            exclude_temporary: compile(config.sqfc().exclude_temporary())?,
        })
    }

    /// The mode for a path relative to the addon folder
    fn mode(&self, path: &str) -> SqfcMode {
        if self.exclude.iter().any(|p| p.matches(path)) {
            SqfcMode::Exclude
        } else if self.exclude_temporary.iter().any(|p| p.matches(path)) {
            SqfcMode::ExcludeTemporary
        } else {
            SqfcMode::Compile
        }
    }
}

/// A file could not be compiled to SQFC
pub struct SqfcFailed {
    file: String,
    reason: String,
}

impl Code for SqfcFailed {
    fn ident(&self) -> &'static str {
        "BSE1"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn message(&self) -> String {
        format!("`{}` could not be compiled to SQFC", self.file)
    }

    fn note(&self) -> Option<String> {
        Some(self.reason.clone())
    }

    fn help(&self) -> Option<String> {
        Some(
            "Add the file to `sqfc.exclude_temporary` in the addon's `addon.toml` to use plain SQF until it can be compiled, \
or to `sqfc.exclude` to never compile it. Splitting large data into multiple files may allow it to be compiled."
                .to_string(),
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

/// A file in `sqfc.exclude_temporary` compiles successfully
pub struct SqfcExcludeNotNeeded {
    file: String,
}

impl Code for SqfcExcludeNotNeeded {
    fn ident(&self) -> &'static str {
        "BSW2"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn message(&self) -> String {
        format!(
            "`{}` is in `sqfc.exclude_temporary` but can be compiled to SQFC",
            self.file
        )
    }

    fn help(&self) -> Option<String> {
        Some("Remove it from `sqfc.exclude_temporary` in the addon's `addon.toml` to include the SQFC".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

#[cfg(test)]
mod tests {
    use super::{SqfcMode, SqfcPatterns};

    fn patterns(exclude: &[&str], exclude_temporary: &[&str]) -> SqfcPatterns {
        let compile = |p: &[&str]| {
            p.iter()
                .map(|p| glob::Pattern::new(p).expect("pattern"))
                .collect()
        };
        SqfcPatterns {
            exclude: compile(exclude),
            exclude_temporary: compile(exclude_temporary),
        }
    }

    #[test]
    fn modes() {
        let patterns = patterns(
            &["functions/fnc_version.sqf", "data/**/*.sqf"],
            &["functions/fnc_large.sqf", "data/big.sqf"],
        );
        assert_eq!(patterns.mode("functions/fnc_other.sqf"), SqfcMode::Compile);
        assert_eq!(
            patterns.mode("functions/fnc_version.sqf"),
            SqfcMode::Exclude
        );
        assert_eq!(patterns.mode("data/nested/a.sqf"), SqfcMode::Exclude);
        assert_eq!(
            patterns.mode("functions/fnc_large.sqf"),
            SqfcMode::ExcludeTemporary
        );
        // `exclude` takes priority
        assert_eq!(patterns.mode("data/big.sqf"), SqfcMode::Exclude);
    }
}
