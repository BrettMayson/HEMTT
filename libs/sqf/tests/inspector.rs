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
    use hemtt_sqf::analyze::inspector::Issue;

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
    #[test]
    #[ignore = "more of a test of the wiki than of hemtt, may break on bad edits to the wiki"]
    pub fn test_wiki_examples() {
        let file = "test_wiki_examples.sqf";
        let mut all_examples = String::new();
        let re = regex::Regex::new(r"<sqf>(.*?)<\/sqf>").expect("regex");
        let database = hemtt_sqf::parser::database::Database::a3(false);
        for (_name, cmd) in database.wiki().commands().iter() {
            if cmd.groups().contains(&String::from("Broken Commands")) {
                continue;
            }
            if [
                "menuenable",                   // Syntax 3 "do not use"
                "execeditorscript",             // "some old editor command"
                "removeCuratorEditableObjects", // Fixed again on wiki
            ]
            .contains(&cmd.name().to_ascii_lowercase().as_str())
            {
                continue;
            }
            for example in cmd.examples() {
                for cap in re.captures_iter(example) {
                    // run each in it's own scope to avoid cross-example issues
                    all_examples.push_str("\n[] spawn {\n");
                    all_examples.push_str(&cap[1]);
                    all_examples.push_str("\n};");
                }
            }
        }

        let folder = std::path::PathBuf::from(crate::ROOT);
        let workspace = hemtt_workspace::Workspace::builder()
            .physical(&folder, hemtt_workspace::LayerType::Source)
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .expect("for test");
        let source = workspace.join(file).expect("for test");
        let mut output = source.create_file().expect("for test");
        output.write_all(all_examples.as_bytes()).expect("for test");

        let processed = hemtt_preprocessor::Processor::run(
            &source,
            &hemtt_common::config::PreprocessorOptions::default(),
        )
        .expect("for test");
        let statements = hemtt_sqf::parser::run(
            &hemtt_sqf::parser::database::Database::a3(false),
            &processed,
        )
        .expect("for test");
        let invalid_args = statements
            .issues()
            .iter()
            .filter(|i| matches!(i, Issue::InvalidArgs(..))) // ignore unused/undefined...
            .collect::<Vec<_>>();
        source.vfs().remove_file().expect("vfs error");
        assert!(invalid_args.is_empty(), "{invalid_args:#?}");
    }
}
