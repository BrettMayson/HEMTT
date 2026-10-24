#![allow(clippy::unwrap_used)]

use std::path::Path;

use hemtt_common::config::{ProjectConfig, RuntimeArguments};
use hemtt_workspace::lint::LintManager;

#[test]
fn extends() {
    let config = ProjectConfig::from_file(Path::new("tests/config/extends.toml")).unwrap();
    let launch = config.hemtt().launch().get("layer2").unwrap();
    assert_eq!(launch.workshop().len(), 9);
    assert_eq!(launch.dlc().len(), 3);
    assert_eq!(launch.presets().len(), 3);
    assert_eq!(launch.optionals().len(), 3);
    assert_eq!(launch.parameters().len(), 3);
    assert_eq!(launch.mission(), Some("base"));
    assert!(!launch.executable().starts_with("arma"));
}

#[test]
fn bad_lint() {
    let project = ProjectConfig::from_file(Path::new("tests/config/bad_lint.toml")).ok();
    let config_lints = project
        .as_ref()
        .map_or_else(Default::default, |project| project.lints().config().clone());
    let runtime = project
        .as_ref()
        .map_or_else(RuntimeArguments::default, |p| p.runtime().clone());
    let manager: LintManager<()> = LintManager::new(config_lints, runtime);
    let codes = manager.check_config_usage("config", "c"); // unordered (HashMap)
    assert_eq!(codes.len(), 1);
}
