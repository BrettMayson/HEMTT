#![allow(clippy::unwrap_used)]

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
    use crate::get_statements;
    use hemtt_sqf::{Expression, Scalar, Statement, UnaryCommand};

    #[test]
    pub fn test_1() {
        let sqf = get_statements("test_1").optimize();
        // println!("debug sqf: {:?}", sqf);

        // -5;
        let Statement::Expression(Expression::Number(value, _), _) = sqf.content()[0].clone()
        else {
            panic!();
        };
        assert_eq!(value, Scalar(-5.0));

        // "A" + "B";
        let Statement::Expression(Expression::String(value, _, _), _) = sqf.content()[1].clone()
        else {
            panic!();
        };
        assert_eq!(*value, *"AB");

        // 1 + 1;
        let Statement::Expression(Expression::Number(value, _), _) = sqf.content()[2].clone()
        else {
            panic!();
        };
        assert_eq!(value, Scalar(2.0));

        // z + z;
        let Statement::Expression(Expression::BinaryCommand(..), _) = sqf.content()[3].clone()
        else {
            panic!();
        };

        // params ["_a", "_b"];
        let Statement::Expression(Expression::UnaryCommand(_, arg_right, _), _) =
            sqf.content()[4].clone()
        else {
            panic!();
        };
        assert!(matches!(*arg_right, Expression::ConsumeableArray(..)));

        // params ["_a", "_b", ["_c", []]];
        let Statement::Expression(Expression::UnaryCommand(_, arg_right, _), _) =
            sqf.content()[5].clone()
        else {
            panic!();
        };
        let Expression::UnaryCommand(UnaryCommand::Plus, arg_plus, ..) = *arg_right else {
            panic!();
        };
        assert!(matches!(*arg_plus, Expression::ConsumeableArray(..)));

        // missionNamespace getVariable ["a", -1];
        let Statement::Expression(Expression::BinaryCommand(_, _, arg_right, _), _) =
            sqf.content()[6].clone()
        else {
            panic!();
        };
        assert!(matches!(*arg_right, Expression::ConsumeableArray(..)));

        // profileNamespace getVariable ["b", [[1]]];
        let Statement::Expression(Expression::BinaryCommand(_, _, arg_right, _), _) =
            sqf.content()[7].clone()
        else {
            panic!();
        };
        let Expression::UnaryCommand(UnaryCommand::Plus, arg_plus, ..) = *arg_right else {
            panic!();
        };
        let Expression::ConsumeableArray(vec, ..) = *arg_plus else {
            panic!();
        };
        assert_eq!(vec.len(), 2);

        // [1,0] vectorAdd p;
        let Statement::Expression(Expression::BinaryCommand(_, arg_left, arg_right, _), _) =
            sqf.content()[8].clone()
        else {
            panic!();
        };
        assert!(matches!(*arg_left, Expression::ConsumeableArray(..)));
        assert!(!matches!(*arg_right, Expression::ConsumeableArray(..)));

        // positionCameraToWorld [10000, 0, 10000];
        let Statement::Expression(Expression::UnaryCommand(_, arg_right, _), _) =
            sqf.content()[9].clone()
        else {
            panic!();
        };
        assert!(matches!(*arg_right, Expression::ConsumeableArray(..)));

        // random [0, _x, 1];
        let Statement::Expression(Expression::UnaryCommand(_, arg_right, _), _) =
            sqf.content()[10].clone()
        else {
            panic!();
        };
        assert!(matches!(*arg_right, Expression::Array(..)));

        assert!(sqf.content().len() == 11);
    }
}
