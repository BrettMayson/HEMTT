use hemtt_common::version::Version;
use rhai::plugin::{
    Dynamic, FnNamespace, FuncRegistration, ImmutableString, Module, NativeCallContext, PluginFunc,
    RhaiResult, TypeId, export_module, mem,
};

use crate::context::Context;

use super::project::RhaiProject;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct RhaiHemtt {
    ctx: Context,
    version: Version,
    project: RhaiProject,
    folder: String,
}

impl RhaiHemtt {
    pub fn new(ctx: &Context) -> Self {
        Self {
            ctx: ctx.clone(),
            version: Version::try_from(env!("HEMTT_VERSION"))
                .expect("hemtt version should be valid"),
            project: RhaiProject::new(ctx),
            folder: ctx.folder().expect("folder exists").to_string(),
        }
    }
}

#[allow(clippy::needless_pass_by_ref_mut)]
#[allow(clippy::unwrap_used)] // coming from rhai codegen
#[export_module]
pub mod project_functions {
    use rhai::EvalAltResult;

    use crate::{
        Error,
        modules::{Hooks, hook::error::bhe1_script_not_found::ScriptNotFound},
        report::Report,
    };

    #[rhai_fn(global, pure)]
    pub fn version(hemtt: &mut RhaiHemtt) -> Version {
        hemtt.version.clone()
    }

    #[rhai_fn(global, pure)]
    pub fn project(hemtt: &mut RhaiHemtt) -> RhaiProject {
        hemtt.project.clone()
    }

    #[rhai_fn(global, pure)]
    pub fn mode(hemtt: &mut RhaiHemtt) -> String {
        hemtt.folder.clone()
    }

    #[rhai_fn(global, pure)]
    pub fn is_dev(hemtt: &mut RhaiHemtt) -> bool {
        hemtt.folder == "dev"
    }

    #[rhai_fn(global, pure)]
    pub fn is_launch(hemtt: &mut RhaiHemtt) -> bool {
        hemtt.folder == "launch"
    }

    #[rhai_fn(global, pure)]
    pub fn is_build(hemtt: &mut RhaiHemtt) -> bool {
        hemtt.folder == "build"
    }

    #[rhai_fn(global, pure)]
    pub fn is_release(hemtt: &mut RhaiHemtt) -> bool {
        hemtt.folder == "release"
    }

    #[rhai_fn(global, pure, return_raw)]
    pub fn script(hemtt: &mut RhaiHemtt, name: &str) -> Result<Dynamic, Box<EvalAltResult>> {
        fn inner_script(hemtt: &mut RhaiHemtt, name: &str) -> Result<(Report, Dynamic), Error> {
            let scripts = hemtt.ctx.workspace_path().join(".hemtt")?.join("scripts")?;
            let path = scripts.join(name)?.with_extension("rhai")?;
            trace!("running script: {}", path.as_str());
            if !path.exists()? {
                return Ok((
                    {
                        let mut report = Report::new();
                        report.push(ScriptNotFound::code(name.to_owned(), &scripts)?);
                        report
                    },
                    Dynamic::UNIT,
                ));
            }
            Hooks::run(&hemtt.ctx, path, false)
        }
        match inner_script(hemtt, name) {
            Ok((report, ret)) => {
                if report.failed() {
                    report.write_to_stdout();
                    Err(Box::new(EvalAltResult::ErrorRuntime(
                        format!("Script '{name}' reported errors").into(),
                        rhai::Position::NONE,
                    )))
                } else {
                    Ok(ret)
                }
            }
            Err(e) => {
                error!("Error running script '{}': {}", name, e);
                Err(Box::new(EvalAltResult::ErrorRuntime(
                    format!("Error running script '{name}': {e}").into(),
                    rhai::Position::NONE,
                )))
            }
        }
    }
}
