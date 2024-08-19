use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
    process::Command,
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc, RwLock,
    },
};

use hemtt_preprocessor::Processor;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use rust_embed::RustEmbed;
use serde::Serialize;
use std::time::Instant;

use crate::{context::Context, error::Error, report::Report};

use super::Module;

#[cfg(windows)]
#[derive(RustEmbed)]
#[folder = "dist/asc/windows"]
struct Distributables;

#[cfg(not(windows))]
#[derive(RustEmbed)]
#[folder = "dist/asc/linux"]
struct Distributables;

#[derive(Default)]
pub struct ArmaScriptCompiler;

#[cfg(windows)]
const SOURCE: [&str; 1] = ["asc.exe"];

#[cfg(not(windows))]
const SOURCE: [&str; 1] = ["asc"];

impl Module for ArmaScriptCompiler {
    fn name(&self) -> &'static str {
        "ArmaScriptCompiler"
    }

    fn init(&mut self, ctx: &Context) -> Result<Report, Error> {
        let asc = ctx.tmp().join("asc");
        trace!("using asc folder at {:?}", asc.display());
        create_dir_all(asc.join("output"))?;
        Ok(Report::new())
    }

    #[allow(clippy::too_many_lines)]
    fn pre_build(&self, ctx: &Context) -> Result<Report, Error> {
        let mut out_file =
            File::create(".hemttout/asc.log").expect("Unable to create `.hemttout/asc.log`");
        let mut config = ASCConfig::new();
        let tmp = ctx.tmp().join("asc");
        for file in SOURCE {
            let out = tmp.join(file);
            let _ = std::fs::create_dir_all(out.parent().expect("must have parent"));
            trace!("unpacking {:?} to {:?}", file, out.display());
            let mut f = File::create(&out)?;
            f.write_all(
                &Distributables::get(file)
                    .expect("dist files should exist")
                    .data,
            )?;
            #[cfg(target_os = "linux")]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(out, PermissionsExt::from_mode(0o744))?;
            }
        }
        let sqf_ext = Some(String::from("sqf"));
        let files = Arc::new(RwLock::new(Vec::new()));
        let start = Instant::now();
        let mut root_dirs = Vec::new();
        for addon in ctx.addons() {
            if !root_dirs.contains(&addon.prefix().main_prefix()) {
                root_dirs.push(addon.prefix().main_prefix());
            }
            let tmp_addon = tmp.join(addon.prefix().as_pathbuf());
            create_dir_all(&tmp_addon)?;
            let mut entries = Vec::new();
            for entry in ctx.workspace_path().join(addon.folder())?.walk_dir()? {
                if entry.is_file()? {
                    if entry.extension() != sqf_ext {
                        continue;
                    }
                    if entry.filename().ends_with(".inc.sqf") {
                        continue;
                    }
                    entries.push(entry);
                }
            }
            entries
                .par_iter()
                .map(|entry| {
                    let processed = Processor::run(entry).map_err(|(_, e)| e)?;
                    let source = tmp_addon.join(
                        entry
                            .as_str()
                            .trim_start_matches(&format!("/{}/", addon.folder())),
                    );
                    let parent = source.parent().expect("must have parent");
                    if !parent.exists() {
                        std::mem::drop(create_dir_all(parent));
                    }
                    let mut f = File::create(source)?;
                    f.write_all(processed.as_str().as_bytes())?;
                    files
                        .write()
                        .expect("unable to write to source files to tmp")
                        .push((
                            format!(
                                "{}{}",
                                addon
                                    .prefix()
                                    .to_string()
                                    .replace('\\', "/")
                                    .trim_end_matches(&addon.folder().replacen(
                                        "optionals/",
                                        "addons/",
                                        1
                                    ))
                                    .trim_end_matches(&addon.folder()),
                                entry.as_str().to_string().trim_start_matches('/'),
                            ),
                            entry
                                .as_str()
                                .to_string()
                                .trim_start_matches('/')
                                .to_string(),
                        ));
                    Ok(())
                })
                .collect::<Result<_, Error>>()?;
        }
        debug!("ASC Preprocess took {:?}", start.elapsed().as_millis());
        for root in root_dirs {
            config.add_input_dir(root.to_string());
        }
        config.set_output_dir(tmp.join("output").display().to_string());
        let include = tmp.join("source").join("include");
        if include.exists() {
            config.add_include_dir(include.display().to_string());
        }
        config.set_worker_threads(num_cpus::get());
        let mut f = File::create(tmp.join("sqfc.json"))?;
        f.write_all(serde_json::to_string_pretty(&config)?.as_bytes())?;
        std::env::set_current_dir(&tmp)?;
        let start = Instant::now();
        let command = Command::new(tmp.join(SOURCE[0])).output()?;
        out_file.write_all(&command.stdout)?;
        out_file.write_all(&command.stderr)?;
        if String::from_utf8(command.stdout.clone())
            .expect("stdout should be valid utf8")
            .contains("Parse Error")
        {
            warn!("ASC 'Parse Error' - check .hemttout/asc.log");
        }
        if command.status.success() {
            debug!("ASC took {:?}", start.elapsed().as_millis());
        } else {
            return Err(Error::ArmaScriptCompiler(
                String::from_utf8(command.stdout).expect("stdout should be valid utf8"),
            ));
        }
        std::env::set_current_dir(ctx.project_folder())?;
        let tmp_output = tmp.join("output");
        let counter = AtomicU16::new(0);
        for (src, dst) in &*files.read().expect("unable to read source files") {
            let from = tmp_output.join(format!("{src}c"));
            let to = ctx.workspace_path().join(format!("{dst}c"))?;
            if !from.exists() {
                // sqf that have parse errors OR just empty//no-code
                debug!("asc didn't process {}", src);
                continue;
            }
            let mut f = File::open(from)?;
            let mut data = Vec::new();
            f.read_to_end(&mut data)?;
            to.create_file()?.write_all(&data)?;
            counter.fetch_add(1, Ordering::Relaxed);
        }
        info!("Compiled {} sqf files", counter.load(Ordering::Relaxed));
        Ok(Report::new())
    }
}

#[derive(Default, Serialize)]
pub struct ASCConfig {
    #[serde(rename = "inputDirs")]
    input_dirs: Vec<String>,
    #[serde(rename = "outputDir")]
    output_dir: String,
    #[serde(rename = "includePaths")]
    include_dirs: Vec<String>,
    #[serde(rename = "excludeList")]
    exclude_list: Vec<String>,
    #[serde(rename = "workerThreads")]
    worker_threads: usize,
}

impl ASCConfig {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            input_dirs: vec![],
            output_dir: String::new(),
            include_dirs: vec![],
            exclude_list: vec![],
            worker_threads: 2,
        }
    }

    pub fn add_input_dir(&mut self, dir: String) {
        if self.input_dirs.contains(&dir) {
            return;
        }
        self.input_dirs.push(dir);
    }

    pub fn set_output_dir(&mut self, dir: String) {
        self.output_dir = dir;
    }

    pub fn add_include_dir(&mut self, dir: String) {
        self.include_dirs.push(dir);
    }

    pub fn add_exclude(&mut self, dir: &str) {
        self.exclude_list.push(dir.replace('/', "\\"));
    }

    pub fn set_worker_threads(&mut self, threads: usize) {
        self.worker_threads = threads;
    }
}
