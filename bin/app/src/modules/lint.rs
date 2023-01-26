use hemtt_preprocessor::preprocess_file;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use super::{preprocessor::VfsResolver, Module};

#[derive(Default)]
pub struct Lint;

impl Module for Lint {
    fn name(&self) -> &'static str {
        "sqf"
    }

    fn check(&self, ctx: &crate::context::Context) -> Result<(), hemtt_bin_error::Error> {
        if ctx.config().asc().enabled() {
            return Ok(());
        }
        if !ctx.config().lint().sqf().enabled() {
            return Ok(());
        }
        for addon in ctx.addons() {
            let resolver = VfsResolver::new(ctx)?;
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
                    if let Err(e) = preprocess_file(entry.as_str(), &resolver) {
                        Err(e.into())
                    } else {
                        Ok(())
                    }
                })
                .collect::<Result<_, hemtt_bin_error::Error>>()?;
        }
        Ok(())
    }
}
