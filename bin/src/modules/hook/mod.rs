use std::{
    ffi::OsString,
    sync::{Arc, Mutex},
};

use ::rhai::{packages::Package, Engine, Scope};

use crate::{context::Context, error::Error};

use self::libraries::hemtt::RhaiHemtt;

use super::Module;

mod libraries;
mod time;

/// Creates a sccope for a Rhai script
///
/// # Errors
/// [`Error::Version`] if the version is not a valid semver version
pub fn scope(ctx: &Context, vfs: bool) -> Result<Scope, Error> {
    let mut scope = Scope::new();
    scope.push_constant("HEMTT_VERSION", env!("CARGO_PKG_VERSION"));
    let version = ctx.config().version().get(ctx.workspace().vfs())?;
    scope.push_constant("HEMTT_PROJECT_VERSION", version.to_string());
    scope.push_constant("HEMTT_PROJECT_VERSION_MAJOR", version.major());
    scope.push_constant("HEMTT_PROJECT_VERSION_MINOR", version.minor());
    scope.push_constant("HEMTT_PROJECT_VERSION_PATCH", version.patch());
    if let Some(build) = version.build() {
        scope.push_constant("HEMTT_PROJECT_VERSION_HASBUILD", true);
        scope.push_constant("HEMTT_PROJECT_VERSION_BUILD", build);
    } else {
        scope.push_constant("HEMTT_PROJECT_VERSION_HASBUILD", false);
    }
    scope.push_constant("HEMTT_PROJECT_NAME", ctx.config().name().to_string());
    scope.push_constant("HEMTT_PROJECT_PREFIX", ctx.config().prefix().to_string());
    scope.push_constant("HEMTT_ADDONS", ctx.addons().to_vec());
    if vfs {
        scope.push_constant("HEMTT_VFS", ctx.workspace().vfs().clone());
    } else {
        scope.push_constant("HEMTT_DIRECTORY", ctx.project_folder().clone());
        scope.push_constant("HEMTT_OUTPUT", ctx.build_folder().clone());
        scope.push_constant("HEMTT_RFS", ctx.project_folder().clone());
        scope.push_constant("HEMTT_OUT", ctx.build_folder().clone());
    }

    scope.push_constant("HEMTT", RhaiHemtt::new(ctx));

    Ok(scope)
}

fn engine(vfs: bool) -> Engine {
    let mut engine = Engine::new();
    if vfs {
        let virt = libraries::VfsPackage::new();
        engine.register_static_module("hemtt_vfs", virt.as_shared_module());
    } else {
        let real = libraries::RfsPackage::new();
        engine.register_static_module("hemtt_rfs", real.as_shared_module());
    }
    engine.register_static_module("hemtt", libraries::HEMTTPackage::new().as_shared_module());
    engine.register_fn("date", time::date);
    engine
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hooks(bool);

impl Default for Hooks {
    fn default() -> Self {
        Self(true)
    }
}

impl Hooks {
    /// Run a folder of hooks
    ///
    /// # Errors
    /// [`Error::ScriptNotFound`] if the script does not exist
    /// [`Error::HookFatal`] if the script calls `fatal`
    /// [`Error::Rhai`] if the script is invalid
    ///
    /// # Panics
    /// If a file path is not a valid [`OsStr`] (UTF-8)
    pub fn run_folder(self, ctx: &Context, name: &str, vfs: bool) -> Result<(), Error> {
        if !self.0 {
            return Ok(());
        }
        let folder = ctx.hemtt_folder().join("hooks").join(name);
        if !folder.exists() {
            trace!("no {} hooks", name);
            return Ok(());
        }
        let mut entries = folder.read_dir()?.collect::<Vec<_>>();
        entries.sort_by_key(|x| {
            x.as_ref()
                .map_or_else(|_| OsString::new(), std::fs::DirEntry::file_name)
        });
        for file in entries {
            let file = file?;
            if !file.file_type()?.is_file() {
                continue;
            }
            info!(
                "Running hook: {}",
                file.path()
                    .display()
                    .to_string()
                    .trim_start_matches(&ctx.hemtt_folder().display().to_string())
            );
            Self::run(
                ctx,
                format!(
                    "{}/{}",
                    name,
                    file.file_name().to_str().expect("Invalid file name")
                ),
                &std::fs::read_to_string(file.path())?,
                vfs,
            )?;
            ctx.config().version().invalidate();
        }
        Ok(())
    }

    /// Run a script
    ///
    /// # Errors
    /// [`Error::ScriptNotFound`] if the script does not exist
    /// [`Error::HookFatal`] if the script calls `fatal`
    /// [`Error::Rhai`] if the script is invalid
    ///
    /// # Panics
    /// If a file path is not a valid [`OsStr`] (UTF-8)
    pub fn run_file(ctx: &Context, name: &str) -> Result<(), Error> {
        let mut path = ctx.hemtt_folder().join("scripts").join(name);
        path.set_extension("rhai");
        if !path.exists() {
            return Err(Error::ScriptNotFound(name.to_owned()));
        }
        let res = Self::run(
            ctx,
            name.to_string(),
            &std::fs::read_to_string(path)?,
            false,
        );
        ctx.config().version().invalidate();
        res
    }

    fn run(ctx: &Context, name: String, script: &str, vfs: bool) -> Result<(), Error> {
        let mut engine = engine(vfs);
        let mut scope = scope(ctx, vfs)?;
        let told_to_fail = Arc::new(Mutex::new(false));
        let inner_name = name.clone();
        engine.on_debug(move |x, _src, _pos| {
            debug!("[{inner_name}] {x}");
        });
        let inner_name = name.clone();
        engine.on_print(move |s| {
            info!("[{inner_name}] {s}");
        });
        let inner_name = name.clone();
        engine.register_fn("info", move |s: &str| {
            info!("[{inner_name}] {s}");
        });
        let inner_name = name.clone();
        engine.register_fn("warn", move |s: &str| {
            warn!("[{inner_name}] {s}");
        });
        let inner_name = name.clone();
        engine.register_fn("error", move |s: &str| {
            error!("[{inner_name}] {s}");
        });
        let inner_name = name.clone();
        let inner_told_to_fail = told_to_fail.clone();
        engine.register_fn("fatal", move |s: &str| {
            error!("[{inner_name}] {s}");
            *inner_told_to_fail.lock().unwrap() = true;
        });
        engine.run_with_scope(&mut scope, script)?;
        if *told_to_fail.lock().unwrap() {
            return Err(Error::HookFatal(name));
        }
        Ok(())
    }
}

impl Module for Hooks {
    fn name(&self) -> &'static str {
        "Hooks"
    }

    fn init(&mut self, ctx: &Context) -> Result<(), Error> {
        self.0 = ctx.hemtt_folder().join("hooks").exists();
        if self.0 {
            for phase in &["pre_build", "post_build", "pre_release", "post_release"] {
                let engine = engine(phase.ends_with("build"));
                let dir = ctx.hemtt_folder().join("hooks").join(phase);
                if !dir.exists() {
                    continue;
                }
                for hook in dir.read_dir().unwrap() {
                    let hook = hook?;
                    engine.compile(&std::fs::read_to_string(hook.path())?)?;
                }
            }
        } else {
            trace!("no hooks folder");
        }
        Ok(())
    }

    fn pre_build(&self, ctx: &Context) -> Result<(), Error> {
        self.run_folder(ctx, "pre_build", true)
    }

    fn post_build(&self, ctx: &Context) -> Result<(), Error> {
        self.run_folder(ctx, "post_build", true)
    }

    fn pre_release(&self, ctx: &Context) -> Result<(), Error> {
        self.run_folder(ctx, "pre_release", false)
    }

    fn post_release(&self, ctx: &Context) -> Result<(), Error> {
        self.run_folder(ctx, "post_release", false)
    }
}
