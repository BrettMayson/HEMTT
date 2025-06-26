use hemtt_common::config::{ProjectConfig, RuntimeArguments};
use hemtt_workspace::{addons::Addon, lint::LintManager, lint_manager, reporting::Codes};

use crate::Project;

pub mod lints {
    mod l02_usage;
    automod::dir!(pub "src/analyze/lints");
}

lint_manager!(stringtable, vec![]);

pub struct LintData {
    pub(crate) addons: Vec<Addon>,
}

pub fn lint_one(
    project: &Project,
    project_config: Option<&ProjectConfig>,
    addons: Vec<Addon>,
) -> Codes {
    let mut manager = LintManager::new(
        project_config.map_or_else(Default::default, |project| {
            project.lints().stringtables().clone()
        }),
        project_config.map_or_else(RuntimeArguments::default, |p| p.runtime().clone()),
    );
    if let Err(e) = manager.extend(
        STRINGTABLE_LINTS
            .iter()
            .map(|l| (**l).clone())
            .collect::<Vec<_>>(),
    ) {
        return e;
    }
    manager.run(&LintData { addons }, project_config, None, project)
}

#[allow(clippy::ptr_arg)] // Needed for &Vec for &dyn Any
#[must_use]
pub fn lint_all(
    projects: &Vec<Project>,
    project_config: Option<&ProjectConfig>,
    addons: Vec<Addon>,
) -> Codes {
    let mut manager = LintManager::new(
        project_config.map_or_else(Default::default, |project| {
            project.lints().stringtables().clone()
        }),
        project_config.map_or_else(RuntimeArguments::default, |p| p.runtime().clone()),
    );
    if let Err(e) = manager.extend(
        STRINGTABLE_LINTS
            .iter()
            .map(|l| (**l).clone())
            .collect::<Vec<_>>(),
    ) {
        return e;
    }
    manager.run(&LintData { addons }, project_config, None, projects)
}
