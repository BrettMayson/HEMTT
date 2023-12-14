use std::path::Path;

use hemtt_common::addons::Addon;

#[test]
#[cfg(not(target_os = "macos"))]
fn case_duplicate() {
    assert_eq!(
        Addon::scan(Path::new("tests/addons_cases"))
            .unwrap_err()
            .to_string(),
        "Addon error: Addon duplicated with different case: Something"
    );
}

#[test]
fn locations_duplicate() {
    assert_eq!(
        Addon::scan(Path::new("tests/addons_duplicate"))
            .unwrap_err()
            .to_string(),
        "Addon error: Addon present in addons and optionals: else"
    );
}
