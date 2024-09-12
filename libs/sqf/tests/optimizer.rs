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

        // toLower "A" + toUpper "b" + toUpperAnsi "C" + toLowerAnsi "d";
        let Statement::Expression(Expression::String(value, _, _), _) = sqf.content()[1].clone()
        else {
            panic!();
        };
        assert_eq!(*value, *"aBCd");

        // 1 + (2 * 2) + (36 % 31) + (36 / 6) + (sqrt 100) - 8;
        let Statement::Expression(Expression::Number(value, _), _) = sqf.content()[2].clone()
        else {
            panic!();
        };
        assert_eq!(value, Scalar(23.0));

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

        // z setVariable ["b", [], true];
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
        assert_eq!(vec.len(), 3);

        // [1,0] vectorAdd p;
        let Statement::Expression(Expression::BinaryCommand(_, arg_left, arg_right, _), _) =
            sqf.content()[8].clone()
        else {
            panic!();
        };
        assert!(matches!(*arg_left, Expression::ConsumeableArray(..)));
        assert!(matches!(*arg_right, Expression::Variable(..)));

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

        // private _z = if (time > 10) then { 1;2;3;4; } else { -1;-2; };
        let Statement::AssignLocal(_, Expression::BinaryCommand(_, _, arg_right, _), _) =
            sqf.content()[11].clone()
        else {
            panic!();
        };
        let Expression::ConsumeableArray(else_vec, _) = *arg_right else {
            panic!();
        };
        assert_eq!(else_vec.len(), 2);
        let (Expression::Code(then_statements), Expression::Code(else_statements)) =
            (else_vec[0].clone(), else_vec[1].clone())
        else {
            panic!();
        };
        assert_eq!(then_statements.content().len(), 4);
        assert_eq!(else_statements.content().len(), 2);

        // sqrt -100;
        let Statement::Expression(
            Expression::UnaryCommand(UnaryCommand::Named(_), arg_right, _),
            _,
        ) = sqf.content()[12].clone()
        else {
            panic!();
        };
        assert!(matches!(*arg_right, Expression::Number(..)));

        // param ["_d"];
        let Statement::Expression(Expression::UnaryCommand(_, arg_right, _), _) =
            sqf.content()[13].clone()
        else {
            panic!();
        };
        assert!(matches!(*arg_right, Expression::ConsumeableArray(..)));

        // [] param ["_e"];
        let Statement::Expression(Expression::BinaryCommand(_, _, arg_right, _), _) =
            sqf.content()[14].clone()
        else {
            panic!();
        };
        assert!(matches!(*arg_right, Expression::ConsumeableArray(..)));

        assert!(sqf.content().len() == 15);
    }
}
