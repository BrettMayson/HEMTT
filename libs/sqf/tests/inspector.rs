use hemtt_sqf::Statements;
use hemtt_workspace::reporting::Processed;

pub use float_ord::FloatOrd as Scalar;
use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::LayerType;
const ROOT: &str = "tests/inspector/";

fn get_statements(file: &str) -> (Processed, Statements, Database) {
    let folder = std::path::PathBuf::from(ROOT);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .expect("for test");
    let source = workspace.join(file).expect("for test");
    let processed = Processor::run(
        &source,
        &hemtt_common::config::PreprocessorOptions::default(),
    )
    .expect("for test");
    let statements = hemtt_sqf::parser::run(&Database::a3(false), &processed).expect("for test");
    let database = Database::a3(false);
    (processed, statements, database)
}

#[cfg(test)]
mod tests {
    use crate::get_statements;

    #[test]
    pub fn test_0() {
        let (_pro, sqf, _database) = get_statements("test_0.sqf");
        // let result = inspector::run_processed(&sqf, &pro, &database);
        let result = sqf.issues();
        println!("done: {}, {result:?}", result.len());
    }
    #[test]
    pub fn test_main() {
        let (_pro, sqf, _database) = get_statements("test_main.sqf");
        let issues = sqf.issues();
        insta::assert_compact_debug_snapshot!((issues.len(), issues));
    }
    #[test]
    pub fn test_optional_args() {
        let (_pro, sqf, _database) = get_statements("test_optional_args.sqf");
        let issues = sqf.issues();
        insta::assert_compact_debug_snapshot!((issues.len(), issues));
    }
    #[test]
    pub fn test_iteration() {
        let (_pro, sqf, _database) = get_statements("test_iteration.sqf");
        let issues = sqf.issues();
        insta::assert_compact_debug_snapshot!((issues.len(), issues));
    }
    #[test]
    pub fn test_variadic() {
        let (_pro, sqf, _database) = get_statements("test_variadic.sqf");
        let issues = sqf.issues();
        insta::assert_compact_debug_snapshot!((issues.len(), issues));
    }
}
