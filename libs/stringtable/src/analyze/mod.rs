use hemtt_common::config::{BuildInfo, ProjectConfig};
use hemtt_workspace::{lint::LintManager, lint_manager, reporting::Codes};
use lints::l01_sorted::StringtableData;

pub mod lints {
    automod::dir!(pub "src/analyze/lints");
}

lint_manager!(stringtable, vec![]);

pub struct LintData {}

pub fn lint_one(
    addon: &StringtableData,
    project: Option<&ProjectConfig>,
    build_info: Option<&BuildInfo>,
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
    manager.run(&LintData {}, project, build_info, None, addon)
}

#[allow(clippy::ptr_arg)] // Needed for &Vec for &dyn Any
pub fn lint_all(
    addons: &Vec<StringtableData>,
    project: Option<&ProjectConfig>,
    build_info: Option<&BuildInfo>,
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
    manager.run(&LintData {}, project, build_info, None, addons)
}
