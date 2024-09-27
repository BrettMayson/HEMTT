use std::sync::Arc;

use hemtt_sqf::Statements;
use hemtt_workspace::reporting::Processed;

pub use float_ord::FloatOrd as Scalar;
use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::LayerType;
const ROOT: &str = "tests/inspector/";

fn get_statements(file: &str) -> (Processed, Statements, Arc<Database>) {
    let folder = std::path::PathBuf::from(ROOT);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join(file).unwrap();
    let processed = Processor::run(&source).unwrap();
    let statements = hemtt_sqf::parser::run(&Database::a3(false), &processed).unwrap();
    let database = Arc::new(Database::a3(false));
    (processed, statements, database)
}

#[cfg(test)]
mod tests {
    use crate::get_statements;
    use hemtt_sqf::analyze::inspector::{self, Issue};

    #[test]
    pub fn test_0() {
        let (pro, sqf, database) = get_statements("test_0.sqf");
        let result = inspector::run_processed(&sqf, &pro, &database, true);
        println!("done: {}, {result:?}", result.len());
    }

    #[test]
    pub fn test_1() {
        let (pro, sqf, database) = get_statements("test_1.sqf");
        let result = inspector::run_processed(&sqf, &pro, &database, true);
        assert_eq!(result.len(), 8);
        // Order not guarenteed
        assert!(result.iter().any(|i| {
            if let Issue::InvalidArgs(cmd, _) = i {
                cmd == "[B:setFuel]"
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::Undefined(var, _, _) = i {
                var == "_guy"
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::NotPrivate(var, _) = i {
                var == "_z"
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::Unused(var, _) = i {
                var == "_c"
            } else {
                false
            }
        }));
        assert!(result.iter().any(|i| {
            if let Issue::Shadowed(var, _) = i {
                var == "_var1"
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
            if let Issue::Undefined(var, _, _) = i {
                var == "_myLocalVar1"
            } else {
                false
            }
        }));
    }
}
