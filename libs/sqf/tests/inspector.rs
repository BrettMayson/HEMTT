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
    use hemtt_sqf::analyze::inspector::{Issue, VarSource};

    #[test]
    pub fn test_0() {
        let (_pro, sqf, _database) = get_statements("test_0.sqf");
        // let result = inspector::run_processed(&sqf, &pro, &database);
        let result = sqf.issues();
        println!("done: {}, {result:?}", result.len());
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    pub fn test_1() {
        let (_pro, sqf, _database) = get_statements("test_1.sqf");
        let result = sqf.issues();
        assert_eq!(result.len(), 16);
        // Order not guarenteed
        assert!(result.iter().any(|i| {
            if let Issue::InvalidArgs(cmd, _) = i {
                cmd == "[B:setFuel]"
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::Undefined(var, _, orphan) = i {
                var == "_test2" && !orphan
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::NotPrivate(var, _) = i {
                var == "_test3"
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::Unused(var, source) = i {
                var == "_test4" && matches!(source, VarSource::Assignment(_, _))
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::Shadowed(var, _) = i {
                var == "_test5"
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::InvalidArgs(var, _) = i {
                var == "[B:addPublicVariableEventHandler]"
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::InvalidArgs(var, _) = i {
                var == "[B:ctrlSetText]"
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::Undefined(var, _, orphan) = i {
                var == "_test8" && !orphan
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::Undefined(var, _, orphan) = i {
                var == "_test9" && !orphan
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::Unused(var, source) = i {
                var == "_test10" && matches!(source, VarSource::ForLoop(_))
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::Unused(var, source) = i {
                var == "_test11" && matches!(source, VarSource::Params(_))
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::InvalidArgs(cmd, _) = i {
                cmd == "[B:drawIcon]"
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::InvalidArgs(cmd, _) = i {
                cmd == "[B:setGusts]"
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::Undefined(var, _, orphan) = i {
                var == "_test12" && !orphan
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::Undefined(var, _, orphan) = i {
                var == "_test13" && *orphan
            } else {
                false
            }
        }));
        assert!(
            result
                .iter()
                .any(|i| { matches!(i, Issue::CountArrayComparison(true, _, _)) })
        );
    }

    #[test]
    pub fn test_2() {
        let (_pro, sqf, _database) = get_statements("test_2.sqf");
        let result = sqf.issues();
        assert_eq!(result.len(), 0);
    }
}
