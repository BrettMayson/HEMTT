use std::sync::atomic::{AtomicU16, Ordering};

use hemtt_preprocessor::Processor;
use hemtt_sqf::{
    analyze::analyze,
    parser::{database::Database, ParserError},
};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{context::Context, error::Error, report::Report};

use super::Module;

#[derive(Default)]
pub struct SQFCompiler;

impl Module for SQFCompiler {
    fn name(&self) -> &'static str {
        "SQF"
    }

    #[allow(clippy::too_many_lines)]
    fn pre_build(&self, ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();
        let sqf_ext = Some(String::from("sqf"));
        let counter = AtomicU16::new(0);
        let mut entries = Vec::new();
        for addon in ctx.addons() {
            for entry in ctx.workspace().join(addon.folder())?.walk_dir()? {
                if entry.is_file()? {
                    if entry.extension() != sqf_ext || entry.filename().ends_with(".inc.sqf") {
                        continue;
                    }
                    entries.push((addon, entry));
                }
            }
        }
        let database = Database::default();
        let reports = entries
            .par_iter()
            .map(|(addon, entry)| {
                trace!("asc compiling {}", entry);
                let processed = Processor::run(entry)?;
                match hemtt_sqf::parser::run(&database, &processed) {
                    Ok(sqf) => {
                        let mut out = entry.with_extension("sqfc")?.create_file()?;
                        let (warnings, errors) =
                            analyze(&sqf, Some(ctx.config()), &processed, addon, &database);
                        let mut report = Report::new();
                        for warning in warnings {
                            report.warn(warning);
                        }
                        if errors.is_empty() {
                            sqf.compile_to_writer(&processed, &mut out)?;
                            counter.fetch_add(1, Ordering::Relaxed);
                        }
                        for error in errors {
                            report.error(error);
                        }
                        Ok(report)
                    }
                    Err(ParserError::ParsingError(e)) => {
                        let mut report = Report::new();
                        if processed.as_str().starts_with("force ")
                            || processed.as_str().contains("\nforce ")
                        {
                            warn!("skipping apparent CBA settings file: {}", entry);
                        } else {
                            for error in e {
                                report.error(error);
                            }
                        }
                        Ok(report)
                    }
                    Err(ParserError::LexingError(e)) => {
                        let mut report = Report::new();
                        for error in e {
                            report.error(error);
                        }
                        Ok(report)
                    }
                }
            })
            .collect::<Result<Vec<Report>, Error>>()?;
        for new_report in reports {
            report.merge(new_report);
        }
        info!("Compiled {} sqf files", counter.load(Ordering::Relaxed));
        Ok(report)
    }
}
