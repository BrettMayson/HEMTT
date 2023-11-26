use std::sync::atomic::{AtomicU16, Ordering};

use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::{database::Database, ParserError};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use time::Instant;

use crate::{context::Context, error::Error};

use super::Module;

#[derive(Default)]
pub struct SQFCompiler;

impl Module for SQFCompiler {
    fn name(&self) -> &'static str {
        "SQF"
    }

    #[allow(clippy::too_many_lines)]
    fn pre_build(&self, ctx: &Context) -> Result<(), Error> {
        let sqf_ext = Some(String::from("sqf"));
        let start = Instant::now();
        let counter = AtomicU16::new(0);
        let mut entries = Vec::new();
        for addon in ctx.addons() {
            for entry in ctx.workspace().join(addon.folder())?.walk_dir()? {
                if entry.is_file()? {
                    if entry.extension() != sqf_ext {
                        continue;
                    }
                    entries.push(entry);
                }
            }
        }
        entries
            .par_iter()
            .map(|entry| {
                trace!("asc compiling {}", entry);
                let processed = Processor::run(entry)?;
                match hemtt_sqf::parser::run(&Database::default(), &processed) {
                    Ok(sqf) => {
                        let mut out = entry.with_extension("sqfc")?.create_file()?;
                        sqf.compile_to_writer(&processed, &mut out)?;
                        counter.fetch_add(1, Ordering::Relaxed);
                        Ok(())
                    }
                    Err(ParserError::ParsingError(e)) => {
                        if entry.filename().ends_with(".inc.sqf") {
                            Ok(())
                        } else if processed.as_str().starts_with("force ")
                            || processed.as_str().contains("\nforce ")
                        {
                            warn!("skipping what appears to be CBA settings file: {}", entry);
                            Ok(())
                        } else {
                            for error in e {
                                eprintln!(
                                    "{}",
                                    error.report_generate_processed(&processed).unwrap()
                                );
                            }
                            Err(Error::Sqf(hemtt_sqf::Error::InvalidSQF))
                        }
                    }
                    Err(ParserError::LexingError(e)) => {
                        if entry.filename().ends_with(".inc.sqf") {
                            Ok(())
                        } else {
                            for error in e {
                                eprintln!("{entry} {error:?}");
                            }
                            Err(Error::Sqf(hemtt_sqf::Error::InvalidSQF))
                        }
                    }
                }
            })
            .collect::<Result<_, Error>>()?;
        debug!(
            "ASC Preprocess took {:?}",
            start.elapsed().whole_milliseconds()
        );
        info!("Compiled {} sqf files", counter.load(Ordering::Relaxed));
        Ok(())
    }
}
