use std::path::Path;

use hemtt_common::addons::Addon;

#[test]
fn locations_duplicate() {
    assert_eq!(
        Addon::scan(Path::new("tests/addons_duplicate"))
            .expect_err("should fail with duplicate locations")
            .to_string(),
        "Addon error: Addon present in addons and optionals: else"
    );
}
