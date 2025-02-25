#![allow(clippy::unwrap_used)]

pub use float_ord::FloatOrd as Scalar;
use hemtt_preprocessor::Processor;
use hemtt_sqf::{parser::database::Database, Statements};
use hemtt_workspace::LayerType;

macro_rules! optimize {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<simple_ $dir>]() {
                insta::assert_debug_snapshot!(optimize(stringify!($dir)));
            }
        }
    };
}

optimize!(consume_array);
optimize!(static_math);
optimize!(scalar);
optimize!(string_case);
optimize!(chain);

const ROOT: &str = "tests/optimizer/";

fn optimize(file: &str) -> Statements {
    let folder = std::path::PathBuf::from(ROOT);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join(format!("{file}.sqf")).unwrap();
    let processed = Processor::run(&source).unwrap();
    hemtt_sqf::parser::run(&Database::a3(false), &processed)
        .unwrap()
        .optimize()
}
