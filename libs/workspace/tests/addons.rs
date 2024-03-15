use std::path::Path;

use hemtt_workspace::addons::Addon;

#[test]
fn locations_duplicate() {
    assert_eq!(
        Addon::scan(Path::new("tests/addons_duplicate"))
            .unwrap_err()
            .to_string(),
        "Addon error: Addon present in addons and optionals: else"
    );
}
