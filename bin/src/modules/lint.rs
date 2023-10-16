use std::sync::atomic::{AtomicU16, Ordering};

use hemtt_common::workspace::WorkspacePath;
use hemtt_preprocessor::Processor;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{context::Context, error::Error};

use super::Module;

#[derive(Default)]
pub struct Lint;

impl Module for Lint {
    fn name(&self) -> &'static str {
        "sqf"
    }

    fn check(&self, ctx: &Context) -> Result<(), Error> {
        if ctx.config().asc().enabled() {
            return Ok(());
        }
        if !ctx.config().lint().sqf().enabled() {
            return Ok(());
        }
        let counter = AtomicU16::new(0);
        for addon in ctx.addons() {
            let sqf_ext = Some(String::from("sqf"));
            let mut entries = Vec::new();
            for entry in ctx.workspace().join(addon.folder())?.walk_dir()? {
                if entry.is_file()? {
                    if entry.extension() != sqf_ext {
                        continue;
                    }
                    if ctx
                        .config()
                        .lint()
                        .sqf()
                        .exclude()
                        .iter()
                        .any(|e| entry.as_str().contains(e))
                    {
                        continue;
                    }
                    entries.push(entry);
                }
            }
            let entry_map = |entry: &WorkspacePath| {
                debug!("linting {:?}", entry.as_str());
                counter.fetch_add(1, Ordering::Relaxed);
                match Processor::run(entry) {
                    Err(e) => Err(e.into()),
                    Ok(processed) => Ok(processed
                        .warnings()
                        .iter()
                        .filter_map(|w| w.report_generate_processed(&processed))
                        .collect::<Vec<String>>()),
                }
            };
            let mut unique_warnings = Vec::new();
            let failed = entries
                .par_iter()
                .map(entry_map)
                .collect::<Vec<Result<Vec<String>, Error>>>()
                .iter()
                .filter(|r| {
                    match r {
                        Err(e) => {
                            error!("{}", e);
                            return true;
                        }
                        Ok(warnings) => {
                            for warning in warnings {
                                if !unique_warnings.contains(warning) {
                                    unique_warnings.push(warning.clone());
                                }
                            }
                        }
                    }
                    false
                })
                .count()
                > 0;
            for warning in unique_warnings {
                eprintln!("{warning}");
            }
            if failed {
                return Err(Error::LintFailed);
            }
        }
        info!("Linted {} files", counter.load(Ordering::Relaxed));
        Ok(())
    }
}
