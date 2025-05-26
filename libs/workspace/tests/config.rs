#![allow(clippy::unwrap_used)]

use std::path::Path;

use hemtt_common::config::ProjectConfig;

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
