use hemtt_workspace::lint::Lint;

pub mod c01_invalid_value;
pub mod c02_duplicate_property;
pub mod c03_duplicate_classes;
pub mod c04_external_missing;
pub mod c05_external_parent_case;
pub mod c06_unexpected_array;
pub mod c07_expected_array;
pub mod c08_missing_semicolon;
pub mod c09_magwell_missing_magazine;

pub fn list() -> Vec<Box<dyn Lint>> {
    vec![
        Box::new(c01_invalid_value::LintC01InvalidValue),
        Box::new(c02_duplicate_property::LintC02DuplicateProperty),
        Box::new(c03_duplicate_classes::LintC03DuplicateClasses),
        Box::new(c04_external_missing::LintC04ExternalMissing),
        Box::new(c05_external_parent_case::LintC05ExternalParentCase),
        Box::new(c06_unexpected_array::LintC06UnexpectedArray),
        Box::new(c07_expected_array::LintC07ExpectedArray),
        Box::new(c08_missing_semicolon::LintC08MissingSemicolon),
        Box::new(c09_magwell_missing_magazine::LintC09MagwellMissingMagazine),
    ]
}
