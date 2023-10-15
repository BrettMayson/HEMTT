use std::{
    collections::HashSet,
    path::PathBuf,
    sync::atomic::{AtomicU16, Ordering},
};

use hemtt_common::workspace::WorkspacePath;
use hemtt_config::{parse, rapify::Rapify};
use hemtt_preprocessor::Processor;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use vfs::VfsFileType;

use crate::{context::Context, error::Error};

use super::Module;

#[derive(Default)]
pub struct Rapifier;

impl Module for Rapifier {
    fn name(&self) -> &'static str {
        "Rapifier"
    }

    fn pre_build(&self, ctx: &Context) -> Result<(), Error> {
        let counter = AtomicU16::new(0);
        let glob_options = glob::MatchOptions {
            require_literal_separator: true,
            ..Default::default()
        };
        let results = ctx
            .addons()
            .par_iter()
            .map(|addon| {
                let mut globs = Vec::new();
                if let Some(config) = addon.config() {
                    if !config.rapify().enabled() {
                        debug!("rapify disabled for {}", addon.name());
                        return Ok((vec![], Ok(())));
                    }
                    for file in config.rapify().exclude() {
                        globs.push(glob::Pattern::new(file)?);
                    }
                }
                let mut messages = Vec::new();
                let mut res = Ok(());
                for entry in ctx.workspace().join(addon.folder())?.walk_dir()? {
                    if entry.metadata()?.file_type == VfsFileType::File
                        && can_rapify(entry.as_str())
                    {
                        if globs
                            .iter()
                            .any(|pat| pat.matches_with(entry.as_str(), glob_options))
                        {
                            debug!("skipping {}", entry.as_str());
                            continue;
                        }
                        debug!("rapifying {}", entry.as_str());
                        let (new_messages, result) = rapify(entry.clone(), ctx);
                        messages.extend(new_messages);
                        counter.fetch_add(1, Ordering::Relaxed);
                        if let Err(e) = result {
                            res = Err(e);
                        }
                    }
                }
                Ok((messages, res))
            })
            .collect::<Result<Vec<(Vec<String>, Result<(), Error>)>, Error>>()?;
        let messages = results.iter().flat_map(|(v, _)| v).collect::<HashSet<_>>();
        for message in messages {
            eprintln!("{message}");
        }
        for (_, result) in results {
            result?;
        }
        info!("Rapified {} addon configs", counter.load(Ordering::Relaxed));
        Ok(())
    }
}

pub fn rapify(path: WorkspacePath, ctx: &Context) -> (Vec<String>, Result<(), Error>) {
    let processed = match Processor::run(&path) {
        Ok(processed) => processed,
        Err(e) => {
            return (Vec::new(), Err(e.into()));
        }
    };
    let mut messages = Vec::new();
    for warning in processed.warnings() {
        if let Some(warning) = warning.generate_report() {
            messages.push(warning);
        }
    }
    let configreport = parse(Some(ctx.config()), &processed);
    if let Err(errors) = configreport {
        for e in &errors {
            eprintln!("{e}");
        }
        return (
            messages,
            Err(Error::Config(hemtt_config::Error::ConfigInvalid(
                path.as_str().to_string(),
            ))),
        );
    };
    let configreport = configreport.unwrap();
    configreport.warnings().iter().for_each(|e| {
        let message = e.generate_processed_report(&processed).map_or_else(
            || {
                e.generate_report()
                    .map_or_else(String::new, |warning| warning)
            },
            |warning| warning,
        );
        if !message.is_empty() {
            messages.push(message);
        }
    });
    configreport.errors().iter().for_each(|e| {
        let message = e.generate_processed_report(&processed).map_or_else(
            || e.generate_report().map_or_else(String::new, |error| error),
            |error| error,
        );
        if !message.is_empty() {
            messages.push(message);
        }
    });
    if !configreport.errors().is_empty() {
        return (
            messages,
            Err(Error::Config(hemtt_config::Error::ConfigInvalid(
                path.as_str().to_string(),
            ))),
        );
    }
    if !configreport.valid() {
        return (
            messages,
            Err(Error::Config(hemtt_config::Error::ConfigInvalid(
                path.as_str().to_string(),
            ))),
        );
    }
    let out = if path.filename().to_lowercase() == "config.cpp" {
        path.parent().join("config.bin").unwrap()
    } else {
        path
    };
    let mut output = match out.create_file() {
        Ok(output) => output,
        Err(e) => {
            return (messages, Err(e.into()));
        }
    };
    if let Err(e) = configreport.config().rapify(&mut output, 0) {
        return (messages, Err(e.into()));
    }
    (messages, Ok(()))
}

pub fn can_rapify(path: &str) -> bool {
    let path = PathBuf::from(path);
    let name = path
        .extension()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_str()
        .unwrap();
    ["cpp", "rvmat", "ext"].contains(&name)
}
