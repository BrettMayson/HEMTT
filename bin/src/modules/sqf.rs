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
pub struct SQFCompiler {
    pub compile: bool,
}

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
            // .par_iter()
            .iter()
            .map(|(addon, entry)| {
                trace!("asc compiling {}", entry);
                let mut report = Report::new();
                let processed = Processor::run(entry)?;
                for warning in processed.warnings() {
                    report.warn(warning.clone());
                }
                match hemtt_sqf::parser::run(&database, &processed) {
                    Ok(sqf) => {
                        let (warnings, errors) =
                            analyze(&sqf, Some(ctx.config()), &processed, Some(addon), &database);
                        for warning in warnings {
                            report.warn(warning);
                        }
                        if errors.is_empty() {
                            if self.compile {
                                let mut out = entry.with_extension("sqfc")?.create_file()?;
                                sqf.compile_to_writer(&processed, &mut out)?;
                                let mut out = entry.with_extension("sqfast")?.create_file()?;
                                out.write_all(format!("{:#?}", sqf.content()).as_bytes())?;
                                let mut out = entry.with_extension("sqfs")?.create_file()?;
                                out.write_all(sqf.source().as_bytes())?;
                            }
                            counter.fetch_add(1, Ordering::Relaxed);
                        }
                        for error in errors {
                            report.error(error);
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
                                report.error(error);
                            }
                        }
                        Ok(report)
                    }
                    Err(ParserError::LexingError(e)) => {
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
        info!(
            "{} {} sqf files",
            if self.compile {
                "Compiled"
            } else {
                "Validated"
            },
            counter.load(Ordering::Relaxed)
        );
        Ok(report)
    }
}
