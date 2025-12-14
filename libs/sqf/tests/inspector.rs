use hemtt_sqf::Statements;
use hemtt_workspace::reporting::Processed;

pub use float_ord::FloatOrd as Scalar;
use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::LayerType;
const ROOT: &str = "tests/inspector/";

macro_rules! inspect {
    ($file:ident) => {
        paste::paste! {
            #[test]
            fn [<$file>]() {
                let (_pro, sqf, _database) = get_statements(stringify!($file.sqf));
                let issues = sqf.issues();
                insta::assert_compact_debug_snapshot!((issues.len(), issues));
            }
        }
    };
}
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
    inspect!(test_main);
    inspect!(test_optional_args);
    inspect!(test_iteration);
    inspect!(test_variadic);
    inspect!(test_code_usage);
    inspect!(test_variable_usage);
    
    #[test]
    #[ignore = "more of a test of the wiki than of hemtt, may break on bad edits to the wiki"]
    pub fn test_wiki_examples() {
        let mut all_examples = String::from("a=1; b=2;"); // gvars get defined in some examples
        let re = regex::Regex::new(r"(?is)<sqf>(.*?)<\/sqf>").expect("regex");
        let database = hemtt_sqf::parser::database::Database::a3(false);
        for (_name, cmd) in database.wiki().commands().iter() {
            if cmd.groups().contains(&String::from("Broken Commands")) {
                continue;
            }
            if [
                "menuenable",         // example 3 "do not use"
                "local",              // example "do not use"
                "sleep",              // example "do not use"
                "execeditorscript",   // "some old editor command"
                "getobjectargument",  // "some old editor command"
                "evalobjectargument", // "some old editor command"
                "isnull",             // example - creatediaryrecord null
                "buttonaction",       // example
                "privateall",         // example
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

        let workspace = hemtt_workspace::Workspace::builder()
            .memory()
            // .physical(
            //     &std::path::PathBuf::from(crate::ROOT),
            //     hemtt_workspace::LayerType::Source,
            // )
            .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
            .expect("for test");
        let source = workspace.join("test_wiki_examples.sqf").expect("for test");
        {
            let mut output = source.create_file().expect("for test");
            output.write_all(all_examples.as_bytes()).expect("for test");
        }

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
            .filter(|i| matches!(i, Issue::InvalidArgs { .. })) // ignore unused/undefined...
            .collect::<Vec<_>>();
        assert!(invalid_args.is_empty(), "{invalid_args:#?}");
    }
}
