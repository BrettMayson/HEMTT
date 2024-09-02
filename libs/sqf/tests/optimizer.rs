pub use float_ord::FloatOrd as Scalar;
use hemtt_preprocessor::Processor;
use hemtt_sqf::{parser::database::Database, Statements};
use hemtt_workspace::LayerType;

const ROOT: &str = "tests/optimizer/";

fn get_statements(dir: &str) -> Statements {
    let folder = std::path::PathBuf::from(ROOT).join(dir);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join("source.sqf").unwrap();
    let processed = Processor::run(&source).unwrap();
    hemtt_sqf::parser::run(&Database::a3(false), &processed).unwrap()
}

#[cfg(test)]
mod tests {
    use hemtt_sqf::{Expression, Scalar, Statement, UnaryCommand};

    use crate::get_statements;

    #[test]
    pub fn test_1() {
        let sqf = get_statements("test_1").optimize();
        println!("sqf: {:?}", sqf);
        assert!(sqf.content().len() == 6);

        {
            // -5;
            let Statement::Expression(e, _) = sqf.content()[0].clone() else {
                panic!();
            };
            let Expression::Number(value, _) = e else {
                panic!();
            };
            assert_eq!(value, Scalar(-5.0));
        }
        {
            // "A" + "B";
            let Statement::Expression(e, _) = sqf.content()[1].clone() else {
                panic!();
            };
            let Expression::String(value, _, _) = e else {
                panic!();
            };
            assert_eq!(*value, *"AB");
        }
        {
            // 1 + 1;
            let Statement::Expression(e, _) = sqf.content()[2].clone() else {
                panic!();
            };
            let Expression::Number(value, _) = e else {
                panic!();
            };
            assert_eq!(value, Scalar(2.0));
        }
        {
            // z + z;
            let Statement::Expression(e, _) = sqf.content()[3].clone() else {
                panic!();
            };
            assert!(matches!(e, Expression::BinaryCommand(..)));
        }
        {
            // params ["_a", "_b"];
            let Statement::Expression(e, _) = sqf.content()[4].clone() else {
                panic!();
            };
            let Expression::UnaryCommand(cmd, arg_right, _) = e else {
                panic!();
            };
            assert!(matches!(cmd, UnaryCommand::Named(..)));
            assert!(matches!(*arg_right, Expression::ConsumeableArray(..)));
        }
        {
            // params ["_a", "_b", ["_c", []]];
            let Statement::Expression(e, _) = sqf.content()[5].clone() else {
                panic!();
            };
            let Expression::UnaryCommand(cmd, arg_right, _) = e else {
                panic!();
            };
            assert!(matches!(cmd, UnaryCommand::Named(..)));
            assert!(matches!(*arg_right, Expression::Array(..)));
        }
    }
}
