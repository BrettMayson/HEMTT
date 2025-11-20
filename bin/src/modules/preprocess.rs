use std::collections::HashMap;

use hemtt_workspace::WorkspacePath;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{context::Context, modules::Module};

#[derive(Debug, Default)]
pub struct PreProcess {
    preprocessors: HashMap<String, String>,
}

impl Module for PreProcess {
    fn name(&self) -> &'static str {
        "preprocess"
    }

    fn priority(&self) -> i32 {
        0
    }

    fn check(&self, ctx: &Context) -> Result<crate::report::Report, crate::Error> {
        for patterns in ctx.config().preprocess().preprocessors().values() {
            for pattern in patterns {
                if let Err(e) = glob::Pattern::new(pattern) {
                    return Err(crate::Error::GlobPattern(e));
                }
            }
        }
        let mut preprocessors = HashMap::new();
        for preprocessor in ctx.config().preprocess().preprocessors().keys() {
            if preprocessors.contains_key(preprocessor) {
                continue;
            }
            let preprocessor_path = ctx
                .workspace_path()
                .join(".hemtt/preprocessors")?
                .join(preprocessor)?;
            if !preprocessor_path.exists()? {
                return Err(crate::Error::PreprocessorNotFound((*preprocessor).clone()));
            }
            preprocessors.insert(preprocessor, preprocessor_path.read_to_string()?);
        }
        Ok(crate::report::Report::new())
    }

    fn pre_build(&self, ctx: &Context) -> Result<crate::report::Report, crate::Error> {
        fn files_to_process(
            ctx: &Context,
        ) -> Result<Vec<(WorkspacePath, Vec<&String>)>, crate::Error> {
            Ok(ctx
                .workspace_path()
                .walk_dir()?
                .into_iter()
                .filter(|e| e.is_file().unwrap_or(false))
                .filter(|e| !e.as_str().contains(".hemttout"))
                .filter_map(|e| {
                    let preprocessors = ctx
                        .config()
                        .preprocess()
                        .preprocessors()
                        .iter()
                        .filter(|(_, ps)| {
                            ps.iter().any(|p| {
                                glob::Pattern::new(p)
                                    .expect("pattern checked")
                                    .matches(e.as_str())
                            })
                        })
                        .collect::<Vec<_>>();
                    if preprocessors.is_empty() {
                        return None;
                    }
                    Some((
                        e,
                        preprocessors
                            .iter()
                            .map(|(p, _)| p.to_owned())
                            .collect::<Vec<_>>(),
                    ))
                })
                .collect::<Vec<_>>())
        }
        let files = files_to_process(ctx)?;
        files
            .par_iter()
            .map(|(path, preprocessors)| {
                let mut content = path.read_to_string()?;
                for preprocessor in preprocessors {
                    let script = self.preprocessors.get(*preprocessor).ok_or_else(|| {
                        crate::Error::PreprocessorNotFound((*preprocessor).to_owned())
                    })?;
                    content = hemtt_rhai::preprocess(script, content, path.as_str().to_string())
                        .expect("preprocessing failed");
                }
                path.create_file()?.write_all(content.as_bytes())?;
                Ok(())
            })
            .collect::<Result<Vec<_>, crate::Error>>()?;
        Ok(crate::report::Report::new())
    }
}
