use std::sync::atomic::{AtomicI32, Ordering};

use hemtt_preprocessor::{preprocess_file, Resolver};
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
        let counter = AtomicI32::new(0);
        for addon in ctx.addons() {
            let resolver = Resolver::new(ctx.vfs(), ctx.prefixes());
            let sqf_ext = Some(String::from("sqf"));
            let mut entries = Vec::new();
            for entry in ctx.vfs().join(addon.folder())?.walk_dir()? {
                let entry = entry?;
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
            entries
                .par_iter()
                .map(|entry| {
                    debug!("linting {:?}", entry.as_str());
                    if let Err(e) = preprocess_file(entry, &resolver) {
                        Err(e.into())
                    } else {
                        counter.fetch_add(1, Ordering::Relaxed);
                        Ok(())
                    }
                })
                .collect::<Result<_, Error>>()?;
        }
        info!("Linted {} files", counter.load(Ordering::Relaxed));
        Ok(())
    }
}
