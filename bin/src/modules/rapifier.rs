use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::atomic::{AtomicU16, Ordering},
};

use hemtt_common::{
    addons::Addon,
    reporting::{Annotation, Code},
    workspace::WorkspacePath,
};
use hemtt_config::{parse, rapify::Rapify};
use hemtt_preprocessor::Processor;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use vfs::VfsFileType;

use crate::{context::Context, error::Error};

use super::Module;

type RapifyResult = (Vec<(String, Vec<Annotation>)>, Result<(), Error>);

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
                        let (new_messages, result) = rapify(addon, entry.clone(), ctx);
                        messages.extend(new_messages);
                        counter.fetch_add(1, Ordering::Relaxed);
                        if let Err(e) = result {
                            res = Err(e);
                        }
                    }
                }
                Ok((messages, res))
            })
            .collect::<Result<Vec<RapifyResult>, Error>>()?;
        let messages = results.iter().flat_map(|(v, _)| v).collect::<HashSet<_>>();
        let mut ci_annotation = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .append(true)
                .open(ctx.out_folder().join("ci_annotation.txt"))?,
        );
        for (message, annotations) in messages {
            eprintln!("{message}");
            for annotation in annotations {
                ci_annotation.write_all(annotation.line().as_bytes())?;
            }
        }
        for (_, result) in results {
            result?;
        }
        info!("Rapified {} addon configs", counter.load(Ordering::Relaxed));
        Ok(())
    }
}

#[allow(clippy::too_many_lines)]
pub fn rapify(addon: &Addon, path: WorkspacePath, ctx: &Context) -> RapifyResult {
    let processed = match Processor::run(&path) {
        Ok(processed) => processed,
        Err(e) => {
            return (Vec::new(), Err(e.into()));
        }
    };
    let mut messages = Vec::new();
    for warning in processed.warnings() {
        if let Some(content) = warning.report_generate() {
            messages.push((content, warning.ci_generate()));
        }
    }
    let configreport = match parse(Some(ctx.config()), &processed) {
        Ok(configreport) => configreport,
        Err(errors) => {
            for e in &errors {
                messages.push((
                    e.report_generate_processed(&processed)
                        .expect("chumsky requires processed"),
                    e.ci_generate_processed(&processed),
                ));
            }
            return (
                messages,
                Err(Error::Config(hemtt_config::Error::ConfigInvalid(
                    path.as_str().to_string(),
                ))),
            );
        }
    };
    configreport.warnings().iter().for_each(|e| {
        let message = e.report_generate_processed(&processed).map_or_else(
            || {
                e.report_generate()
                    .map_or_else(String::new, |warning| warning)
            },
            |warning| warning,
        );
        if !message.is_empty() {
            messages.push((message, {
                let mut annotations = e.ci_generate_processed(&processed);
                annotations.extend(e.ci_generate());
                annotations
            }));
        }
    });
    configreport.errors().iter().for_each(|e| {
        let message = e.report_generate_processed(&processed).map_or_else(
            || e.report_generate().map_or_else(String::new, |error| error),
            |error| error,
        );
        if !message.is_empty() {
            messages.push((message, {
                let mut annotations = e.ci_generate_processed(&processed);
                annotations.extend(e.ci_generate());
                annotations
            }));
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
        let (version, cfgpatch) = configreport.required_version();
        let mut file = path.as_str().to_string();
        let mut span = 0..0;
        if let Some(cfgpatch) = cfgpatch {
            let map = processed.mapping(cfgpatch.name().span.start).unwrap();
            file = map.original().path().as_str().to_string();
            span = map.original().start().0..map.original().end().0;
        }
        addon.build_data().set_required_version(version, file, span);
        path.parent().join("config.bin").unwrap()
    } else {
        path
    };
    if processed.no_rapify() {
        debug!(
            "skipping rapify for {}, as instructed by preprocessor",
            out.as_str()
        );
        return (messages, Ok(()));
    }
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
