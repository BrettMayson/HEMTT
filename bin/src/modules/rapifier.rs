use std::{collections::HashMap, path::PathBuf, sync::RwLock};

use hemtt_config::{
    Config,
    analyze::{lint_all, lint_check},
    parse,
    rapify::Rapify,
};
use hemtt_preprocessor::Processor;
use hemtt_workspace::{
    WorkspacePath,
    addons::{Addon, Location},
};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use vfs::VfsFileType;

use crate::{context::Context, error::Error, progress::progress_bar, report::Report};

use super::Module;

type InnerAddonConfig = RwLock<HashMap<(String, Location), Vec<(WorkspacePath, Config)>>>;

#[derive(Default)]
pub struct AddonConfigs(InnerAddonConfig);

impl std::ops::Deref for AddonConfigs {
    type Target = InnerAddonConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct Rapifier;

impl Module for Rapifier {
    fn name(&self) -> &'static str {
        "Rapifier"
    }
    fn priority(&self) -> i32 {
        2000
    }

    fn check(&self, ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();
        report.extend(lint_check(
            ctx.config().lints().config().clone(),
            ctx.config().runtime().clone(),
        ));
        Ok(report)
    }

    fn pre_build(&self, ctx: &Context) -> Result<Report, Error> {
        ctx.state().set(AddonConfigs::default());
        let mut report = Report::new();
        let glob_options = glob::MatchOptions {
            require_literal_separator: true,
            ..Default::default()
        };
        let mut entries = Vec::new();
        ctx.addons()
            .iter()
            .map(|addon| {
                let mut globs = Vec::new();
                if let Some(config) = addon.config() {
                    if !config.rapify().enabled() {
                        debug!("rapify disabled for {}", addon.name());
                        return Ok(());
                    }
                    for file in config.rapify().exclude() {
                        globs.push(glob::Pattern::new(file)?);
                    }
                }
                for entry in ctx.workspace_path().join(addon.folder())?.walk_dir()? {
                    if entry.metadata()?.file_type == VfsFileType::File && can_rapify(&entry)? {
                        if globs
                            .iter()
                            .any(|pat| pat.matches_with(entry.as_str(), glob_options))
                        {
                            debug!("skipping {}", entry.as_str());
                            continue;
                        }
                        entries.push((addon, entry));
                    }
                }
                Ok(())
            })
            .collect::<Result<Vec<_>, Error>>()?;

        let progress = progress_bar(entries.len() as u64).with_message("Rapifying Configs");
        let reports = entries
            .par_iter()
            .map(|(addon, entry)| {
                let report = rapify(addon, entry, ctx)?;
                progress.inc(1);
                Ok(report)
            })
            .collect::<Result<Vec<Report>, Error>>()?;

        for new_report in reports {
            report.merge(new_report);
        }

        progress.finish_and_clear();
        info!("Rapified {} addon configs", entries.len());
        report.extend(lint_all(Some(ctx.config()), &ctx.addons().to_vec()));
        Ok(report)
    }
}

#[allow(clippy::too_many_lines)]
pub fn rapify(addon: &Addon, path: &WorkspacePath, ctx: &Context) -> Result<Report, Error> {
    let mut report = Report::new();
    let processed = match Processor::run(path, ctx.config().preprocessor()) {
        Ok(processed) => processed,
        Err((_, hemtt_preprocessor::Error::Code(e))) => {
            report.push(e);
            return Ok(report);
        }
        Err((_, e)) => {
            return Err(e.into());
        }
    };
    for warning in processed.warnings() {
        report.push(warning.clone());
    }
    let configreport = match parse(Some(ctx.config()), &processed) {
        Ok(configreport) => configreport,
        Err(errors) => {
            for e in &errors {
                report.push(e.clone());
            }
            return Ok(report);
        }
    };
    configreport.push_to_addon(addon);
    configreport.notes_and_helps().into_iter().for_each(|e| {
        report.push(e.clone());
    });
    configreport.warnings().into_iter().for_each(|e| {
        report.push(e.clone());
    });
    configreport.errors().into_iter().for_each(|e| {
        report.push(e.clone());
    });
    if !configreport.errors().is_empty() {
        return Ok(report);
    }
    let out = if std::path::Path::new(&path.filename())
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("cpp"))
    {
        if path.filename() == "config.cpp" {
            let (version, cfgpatch) = configreport.required_version();
            let mut file = path;
            let span = cfgpatch.map_or(0..0, |cfgpatch| {
                let map = processed
                    .mapping(cfgpatch.name().span.start)
                    .expect("mapping should exist");
                file = map.original().path();
                map.original().start().0..map.original().end().0
            });
            addon
                .build_data()
                .set_required_version(version, file.to_owned(), span);
            ctx.state()
                .get::<AddonConfigs>()
                .write()
                .expect("state is poisoned")
                .entry((addon.name().to_owned(), *addon.location()))
                .or_default()
                .push((file.to_owned(), configreport.config().clone()));
        }
        path.with_extension("bin")?
    } else {
        path.to_owned()
    };
    if processed.no_rapify() {
        debug!(
            "skipping rapify for {}, as instructed by preprocessor",
            out.as_str()
        );
        return Ok(report);
    }
    let mut output = match out.create_file() {
        Ok(output) => output,
        Err(e) => {
            return Err(e.into());
        }
    };
    if let Err(e) = configreport.config().rapify(&mut output, 0) {
        return Err(e.into());
    }
    Ok(report)
}

pub fn can_rapify(entry: &WorkspacePath) -> Result<bool, Error> {
    let path = entry.as_str();
    let pathbuf = PathBuf::from(&path);
    let ext = pathbuf
        .extension()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_str()
        .expect("osstr should be valid utf8");
    if ext == "cpp" && pathbuf.file_name() != Some(std::ffi::OsStr::new("config.cpp")) {
        warn!(
            "{} - cpp files other than config.cpp are usually not intentional. use hpp for includes",
            path.trim_start_matches('/')
        );
    }
    if !["cpp", "rvmat", "ext", "sqm", "bikb", "bisurf"].contains(&ext) {
        return Ok(false);
    }
    let mut buffer = vec![0; 4];
    if entry.open_file()?.read_exact(&mut buffer).is_err() {
        // The file is less than 4 bytes, so it is not rapified
        return Ok(true);
    }
    Ok(buffer != b"\0raP")
}
