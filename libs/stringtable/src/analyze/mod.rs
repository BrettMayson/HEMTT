use hemtt_common::config::ProjectConfig;
use hemtt_workspace::{
    addons::Addon, lint::LintManager, lint_manager, reporting::Codes, WorkspacePath,
};

pub mod lints {
    automod::dir!(pub "src/analyze/lints");
}

lint_manager!(stringtable, vec![]);

pub struct SqfLintData {
    workspace: WorkspacePath,
}

impl SqfLintData {
    #[must_use]
    pub const fn workspace(&self) -> &WorkspacePath {
        &self.workspace
    }
}

pub fn lint_addon(
    workspace: WorkspacePath,
    addon: &Addon,
    project: Option<&ProjectConfig>,
) -> Codes {
    let mut manager = LintManager::new(project.map_or_else(Default::default, |project| {
        project.lints().stringtables().clone()
    }));
    if let Err(e) = manager.extend(
        STRINGTABLE_LINTS
            .iter()
            .map(|l| (**l).clone())
            .collect::<Vec<_>>(),
    ) {
        return e;
    }
    manager.run(&SqfLintData { workspace }, project, None, addon)
}

#[allow(clippy::ptr_arg)] // Needed for &Vec for &dyn Any
pub fn lint_addons(
    workspace: WorkspacePath,
    addons: &Vec<Addon>,
    project: Option<&ProjectConfig>,
) -> Codes {
    let mut manager = LintManager::new(project.map_or_else(Default::default, |project| {
        project.lints().stringtables().clone()
    }));
    if let Err(e) = manager.extend(
        STRINGTABLE_LINTS
            .iter()
            .map(|l| (**l).clone())
            .collect::<Vec<_>>(),
    ) {
        return e;
    }
    manager.run(&SqfLintData { workspace }, project, None, addons)
}
