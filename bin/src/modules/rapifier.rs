use std::{
    path::PathBuf,
    sync::atomic::{AtomicI16, Ordering},
};

use hemtt_config::{parse, rapify::Rapify};
use hemtt_preprocessor::{preprocess_file, Resolver};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use vfs::{VfsFileType, VfsPath};

use crate::{context::Context, error::Error};

use super::Module;

#[derive(Default)]
pub struct Rapifier;

impl Module for Rapifier {
    fn name(&self) -> &'static str {
        "Rapifier"
    }

    fn pre_build(&self, ctx: &Context) -> Result<(), Error> {
        let resolver = Resolver::new(ctx.vfs(), ctx.prefixes());
        let counter = AtomicI16::new(0);
        let glob_options = glob::MatchOptions {
            require_literal_separator: true,
            ..Default::default()
        };
        ctx.addons()
            .par_iter()
            .map(|addon| {
                let mut globs = Vec::new();
                if let Some(config) = addon.config() {
                    if !config.preprocess().enabled() {
                        debug!("preprocessing disabled for {}", addon.name());
                        return Ok(());
                    }
                    for file in config.preprocess().exclude() {
                        globs.push(glob::Pattern::new(file)?);
                    }
                }
                for entry in ctx.vfs().join(addon.folder())?.walk_dir()? {
                    let entry = entry?;
                    if entry.metadata()?.file_type == VfsFileType::File
                        && can_preprocess(entry.as_str())
                    {
                        if globs
                            .iter()
                            .any(|pat| pat.matches_with(entry.as_str(), glob_options))
                        {
                            debug!("skipping {}", entry.as_str());
                            continue;
                        }
                        debug!("rapifying {}", entry.as_str());
                        rapify(entry.clone(), ctx, &resolver)?;
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Ok(())
            })
            .collect::<Result<(), Error>>()?;
        info!("Rapified {} addon configs", counter.load(Ordering::Relaxed));
        Ok(())
    }
}

pub fn rapify(path: VfsPath, _ctx: &Context, resolver: &Resolver) -> Result<(), Error> {
    let processed = preprocess_file(&path, resolver)?;
    let configreport = parse(&processed);
    if let Err(errors) = configreport {
        for e in &errors {
            eprintln!("{e}");
        }
        return Err(Error::Config(hemtt_config::Error::ConfigInvalid(
            path.as_str().to_string(),
        )));
    };
    let configreport = configreport.unwrap();
    configreport.warnings().iter().for_each(|e| {
        e.generate_processed_report(&processed).map_or_else(|| if let Some(warning) = e.generate_report() {
            eprintln!("{warning}");
        }, |warning| {
            eprintln!("{warning}");
        });
    });
    configreport.errors().iter().for_each(|e| {
        e.generate_processed_report(&processed).map_or_else(|| if let Some(error) = e.generate_report() {
            eprintln!("{error}");
        }, |error| {
            eprintln!("{error}");
        });
    });
    if !configreport.errors().is_empty() {
        return Err(Error::Config(hemtt_config::Error::ConfigInvalid(
            path.as_str().to_string(),
        )));
    }
    if !configreport.valid() {
        return Err(Error::Config(hemtt_config::Error::ConfigInvalid(
            path.as_str().to_string(),
        )));
    }
    let out = if path.filename() == "config.cpp" {
        path.parent().join("config.bin").unwrap()
    } else {
        path
    };
    let mut output = out.create_file()?;
    configreport.config().rapify(&mut output, 0)?;
    Ok(())
}

pub fn can_preprocess(path: &str) -> bool {
    let path = PathBuf::from(path);
    let name = path
        .extension()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_str()
        .unwrap();
    ["cpp", "rvmat", "ext"].contains(&name)
}
