use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
    process::Command,
};

use hemtt_preprocessor::preprocess_file;
use rust_embed::RustEmbed;
use serde::Serialize;

use super::{preprocessor::VfsResolver, Module};

#[cfg(windows)]
#[derive(RustEmbed)]
#[folder = "dist/asc/windows"]
struct Distributables;

#[cfg(not(windows))]
#[derive(RustEmbed)]
#[folder = "dist/asc/linux"]
struct Distributables;

pub struct ArmaScriptCompiler;
impl ArmaScriptCompiler {
    pub const fn new() -> Self {
        Self
    }
}

#[cfg(windows)]
const SOURCE: [&str; 1] = ["asc.exe"];

#[cfg(not(windows))]
const SOURCE: [&str; 1] = ["asc"];

impl Module for ArmaScriptCompiler {
    fn name(&self) -> &'static str {
        "ArmaScriptCompiler"
    }

    fn init(&mut self, ctx: &crate::context::Context) -> Result<(), hemtt_bin_error::Error> {
        if !ctx.config().asc().enabled() {
            return Ok(());
        }
        create_dir_all(ctx.tmp().join("asc").join("output"))?;
        Ok(())
    }

    fn check(&self, ctx: &crate::context::Context) -> Result<(), hemtt_bin_error::Error> {
        if !ctx.config().asc().enabled() {
            return Ok(());
        }
        for exclude in ctx.config().asc().exclude() {
            if exclude.contains('*') {
                return Err(hemtt_bin_error::Error::ASC(
                    "wildcards are not supported".to_string(),
                ));
            }
            if exclude.contains('\\') {
                return Err(hemtt_bin_error::Error::ASC(
                    "backslashes are not supported, use forward slashes".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn pre_build(&self, ctx: &crate::context::Context) -> Result<(), hemtt_bin_error::Error> {
        if !ctx.config().asc().enabled() {
            return Ok(());
        }
        let mut config = ASCConfig::new();
        let tmp = ctx.tmp().join("asc");
        for file in SOURCE {
            let mut f = File::create(tmp.join(file))?;
            f.write_all(&Distributables::get(file).unwrap().data)?;
            #[cfg(target_os = "linux")]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = f.metadata()?;
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o744);
            }
        }
        let resolver = VfsResolver::new(ctx)?;
        let sqf_ext = Some(String::from("sqf"));
        let mut files = Vec::new();
        for addon in ctx.addons() {
            let tmp_addon = tmp.join("source").join(addon.folder());
            create_dir_all(&tmp_addon)?;
            for entry in ctx.vfs().join(addon.folder())?.walk_dir()? {
                let entry = entry?;
                if entry.is_file()? {
                    if entry.extension() != sqf_ext {
                        continue;
                    }
                    if ctx
                        .config()
                        .asc()
                        .exclude()
                        .iter()
                        .any(|e| entry.as_str().contains(e))
                    {
                        continue;
                    }
                    let tokens = preprocess_file(entry.as_str(), &resolver)?;
                    let source = tmp_addon.join(
                        entry
                            .as_str()
                            .trim_start_matches(&format!("/{}/", addon.folder())),
                    );
                    let _ = create_dir_all(source.parent().unwrap());
                    let mut f = File::create(source)?;
                    for t in tokens {
                        f.write_all(t.to_source().as_bytes())?;
                    }
                    files.push(
                        entry
                            .as_str()
                            .to_string()
                            .trim_start_matches('/')
                            .to_string(),
                    );
                }
            }
        }
        config.add_input_dir(tmp.join("source").display().to_string());
        config.set_output_dir(tmp.join("output").display().to_string());
        let include = tmp.join("source").join("include");
        if include.exists() {
            config.add_include_dir(include.display().to_string());
        }
        for exclude in ctx.config().asc().exclude() {
            config.add_exclude(exclude);
        }
        config.set_worker_threads(num_cpus::get());
        let mut f = File::create(tmp.join("sqfc.json"))?;
        f.write_all(serde_json::to_string_pretty(&config)?.as_bytes())?;
        let old_dir = std::env::current_dir()?;
        std::env::set_current_dir(&tmp)?;
        let command = Command::new(tmp.join(SOURCE[0])).output()?;
        if command.status.success() {
            println!("Compiled sqf files");
        } else {
            return Err(hemtt_bin_error::Error::ASC(
                String::from_utf8(command.stdout).unwrap(),
            ));
        }
        std::env::set_current_dir(old_dir)?;
        let tmp_output = tmp.join("output").join(
            tmp.join("source")
                .display()
                .to_string()
                .trim_start_matches(&tmp.ancestors().last().unwrap().display().to_string()),
        );
        for file in files {
            let file = format!("{file}c");
            let from = tmp_output.join(&file);
            let to = ctx.vfs().join(&file)?;
            if !from.exists() {
                // Likely excluded
                continue;
            }
            let mut f = File::open(from)?;
            let mut data = Vec::new();
            f.read_to_end(&mut data)?;
            to.create_file()?.write_all(&data)?;
        }
        Ok(())
    }
}

#[derive(Default, Serialize)]
struct ASCConfig {
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
