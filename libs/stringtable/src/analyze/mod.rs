use hemtt_common::config::ProjectConfig;
use hemtt_workspace::{lint::LintManager, lint_manager, reporting::Codes};
use lints::_01_sorted::StringtableData;

pub mod lints {
    automod::dir!(pub "src/analyze/lints");
}

lint_manager!(stringtable, vec![]);

pub struct SqfLintData {}

pub fn lint_addon(addon: &StringtableData, project: Option<&ProjectConfig>) -> Codes {
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
    manager.run(&SqfLintData {}, project, None, addon)
}

#[allow(clippy::ptr_arg)] // Needed for &Vec for &dyn Any
pub fn lint_addons(addons: &Vec<StringtableData>, project: Option<&ProjectConfig>) -> Codes {
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
    manager.run(&SqfLintData {}, project, None, addons)
}
