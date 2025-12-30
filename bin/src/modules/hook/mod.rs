use ::rhai::{Engine, Scope, packages::Package};
use hemtt_rhai::libraries::VfsPackage;
use hemtt_workspace::WorkspacePath;
use rhai::Dynamic;
use whoami::Result;

use crate::{
    context::Context,
    error::Error,
    modules::hook::{error::bhe5_runtime_fatal::RuntimeFatal, libraries::hemtt::RhaiHemtt},
    report::Report,
};

use self::error::{
    bhe1_script_not_found::ScriptNotFound, bhe3_parse_error::RhaiParseError,
    bhe4_runtime_error::RuntimeError,
};

use super::Module;

mod error;
mod libraries;

/// Creates a sccope for a Rhai script
///
/// # Errors
/// [`Error::Version`] if the version is not a valid semver version
///
/// # Panics
/// If the build folder does not exist
pub fn scope(ctx: &'_ Context, hemtt: bool, vfs: bool) -> Result<Scope<'_>, Error> {
    let mut scope = Scope::new();
    if vfs {
        scope.push_constant("HEMTT_VFS", ctx.workspace_path().vfs().clone());
    }
    scope.push_constant("HEMTT_RFS", ctx.project_folder().clone());

    if hemtt {
        scope.push_constant(
            "HEMTT_OUTPUT",
            ctx.build_folder().expect("build folder exists").clone(),
        );
        scope.push_constant(
            "HEMTT_OUT",
            ctx.build_folder().expect("build folder exists").clone(),
        );
        scope.push_constant("HEMTT", RhaiHemtt::new(ctx));
    }

    Ok(scope)
}

fn engine(name: String, vfs: bool) -> Engine {
    let mut engine = hemtt_rhai::engine(name);
    if vfs {
        let virt = VfsPackage::new();
        engine.register_static_module("hemtt_vfs", virt.as_shared_module());
    }
    engine.register_static_module("hemtt", libraries::HEMTTPackage::new().as_shared_module());
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
    pub fn run_folder(self, ctx: &Context, name: &str, vfs: bool) -> Result<Report, Error> {
        if !self.0 {
            return Ok(Report::new());
        }
        let folder = ctx
            .workspace_path()
            .join(".hemtt")?
            .join("hooks")?
            .join(name)?;
        if !folder.exists()? {
            trace!("no {} hooks", name);
            return Ok(Report::new());
        }
        let mut entries = folder.read_dir()?;
        entries.sort_by_key(WorkspacePath::filename);
        let mut report = Report::new();
        for file in entries {
            if !file.is_file()? {
                continue;
            }
            info!(
                "Running hook: {}",
                file.as_str().trim_start_matches("/.hemtt/hooks/")
            );
            report.merge(Self::run(ctx, file, vfs)?.0);
            ctx.config().version().invalidate();
            if report.failed() {
                break;
            }
        }
        Ok(report)
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
    pub fn run_file(ctx: &Context, name: &str) -> Result<(Report, Dynamic), Error> {
        let mut report = Report::new();
        let scripts = ctx.workspace_path().join(".hemtt")?.join("scripts")?;
        let path = scripts.join(name)?.with_extension("rhai")?;
        trace!("running script: {}", path.as_str());
        if !path.exists()? {
            report.push(ScriptNotFound::code(name.to_owned(), &scripts)?);
            return Ok((report, Dynamic::UNIT));
        }
        let res = Self::run(ctx, path, false);
        ctx.config().version().invalidate();
        res
    }

    #[allow(clippy::needless_pass_by_value)] // rhai things
    fn run(ctx: &Context, path: WorkspacePath, vfs: bool) -> Result<(Report, Dynamic), Error> {
        let mut report = Report::new();
        let parts = path.as_str().split('/');
        let name = parts
            .clone()
            .skip(parts.count().saturating_sub(2))
            .collect::<Vec<_>>()
            .join("/");
        let engine = engine(name, vfs);
        let mut scope = scope(ctx, true, vfs)?;
        match engine.eval_with_scope(&mut scope, &path.read_to_string()?) {
            Err(e) => {
                match *e {
                    rhai::EvalAltResult::ErrorTerminated(message, pos) => {
                        report.push(RuntimeFatal::code(path, message.to_string(), pos));
                    }
                    _ => {
                        report.push(RuntimeError::code(path, &e));
                    }
                }
                Ok((report, Dynamic::UNIT))
            }
            Ok(ret) => Ok((report, ret)),
        }
    }
}

impl Module for Hooks {
    fn name(&self) -> &'static str {
        "Hooks"
    }

    fn init(&mut self, ctx: &Context) -> Result<Report, Error> {
        let mut report = Report::new();
        self.0 = ctx.hemtt_folder().join("hooks").exists();
        if self.0 {
            for phase in &["pre_build", "post_build", "pre_release", "post_release"] {
                let engine = engine(String::from(*phase), phase != &"post_release");
                let dir = ctx.hemtt_folder().join("hooks").join(phase);
                if !dir.exists() {
                    continue;
                }
                for hook in fs_err::read_dir(&dir).expect("hooks folder should be readable") {
                    let hook = hook?;
                    let path = ctx
                        .workspace_path()
                        .join(".hemtt")?
                        .join("hooks")?
                        .join(phase)?
                        .join(hook.file_name().to_str().expect("file name is valid utf-8"))?;
                    if let Err(e) = engine.compile(&path.read_to_string()?) {
                        report.push(RhaiParseError::code(path, e.0, e.1));
                    }
                }
            }
        } else {
            trace!("no hooks folder");
        }
        Ok(report)
    }

    fn pre_build(&self, ctx: &Context) -> Result<Report, Error> {
        self.run_folder(ctx, "pre_build", true)
    }

    fn post_build(&self, ctx: &Context) -> Result<Report, Error> {
        self.run_folder(ctx, "post_build", true)
    }

    fn pre_release(&self, ctx: &Context) -> Result<Report, Error> {
        self.run_folder(ctx, "pre_release", true)
    }

    fn archive(&self, ctx: &Context) -> Result<Report, Error> {
        self.run_folder(ctx, "archive", false)
    }

    fn post_release(&self, ctx: &Context) -> Result<Report, Error> {
        self.run_folder(ctx, "post_release", false)
    }
}
