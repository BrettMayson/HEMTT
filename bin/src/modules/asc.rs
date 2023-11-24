use std::sync::atomic::{AtomicU16, Ordering};

use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::{database::Database, ParserError};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use time::Instant;

use crate::{context::Context, error::Error};

use super::Module;

#[derive(Default)]
pub struct ArmaScriptCompiler;

impl Module for ArmaScriptCompiler {
    fn name(&self) -> &'static str {
        "ArmaScriptCompiler"
    }

    fn init(&mut self, ctx: &Context) -> Result<(), Error> {
        if !ctx.config().asc().enabled() {
            trace!("disabled");
            return Ok(());
        }
        Ok(())
    }

    fn check(&self, ctx: &Context) -> Result<(), Error> {
        if !ctx.config().asc().enabled() {
            return Ok(());
        }
        for exclude in ctx.config().asc().exclude() {
            if exclude.contains('*') {
                return Err(Error::ArmaScriptCompiler(
                    "wildcards are not supported".to_string(),
                ));
            }
            if exclude.contains('\\') {
                return Err(Error::ArmaScriptCompiler(
                    "backslashes are not supported, use forward slashes".to_string(),
                ));
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn pre_build(&self, ctx: &Context) -> Result<(), Error> {
        if !ctx.config().asc().enabled() {
            return Ok(());
        }
        let sqf_ext = Some(String::from("sqf"));
        let start = Instant::now();
        let counter = AtomicU16::new(0);
        for addon in ctx.addons() {
            let mut entries = Vec::new();
            for entry in ctx.workspace().join(addon.folder())?.walk_dir()? {
                if entry.is_file()? {
                    if entry.extension() != sqf_ext {
                        continue;
                    }
                    if ctx.config().asc().exclude().iter().any(|e| {
                        entry
                            .as_str()
                            .to_ascii_lowercase()
                            .contains(&e.to_ascii_lowercase())
                    }) {
                        debug!("asc excluded {}", entry);
                        continue;
                    }
                    entries.push(entry);
                }
            }
            entries
                .par_iter()
                .map(|entry| {
                    trace!("asc compiling {}", entry);
                    let processed = Processor::run(entry)?;
                    let sqf = match hemtt_sqf::parser::run(&Database::default(), &processed) {
                        Ok(sqf) => sqf,
                        Err(ParserError::ParsingError(e)) => {
                            for error in e {
                                eprintln!(
                                    "{}",
                                    error.report_generate_processed(&processed).unwrap()
                                );
                            }
                            return Ok(());
                        }
                        Err(e) => return Err(Error::Sqf(e.into())),
                    };
                    let mut out = entry.with_extension("sqfc")?.create_file()?;
                    sqf.compile_to_writer(&processed, &mut out)?;
                    counter.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
                .collect::<Result<_, Error>>()?;
        }
        debug!(
            "ASC Preprocess took {:?}",
            start.elapsed().whole_milliseconds()
        );
        info!("Compiled {} sqf files", counter.load(Ordering::Relaxed));
        Ok(())
    }
}
