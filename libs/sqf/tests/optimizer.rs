#![allow(clippy::unwrap_used)]

pub use float_ord::FloatOrd as Scalar;
use hemtt_preprocessor::Processor;
use hemtt_sqf::{Statements, parser::database::Database};
use hemtt_workspace::{LayerType, WorkspacePath, reporting::Processed};

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
optimize!(select);
optimize!(string_case);
optimize!(chain);
optimize!(to_string);
optimize!(array_all_positives);
optimize!(array_all_negatives);
optimize!(if_not_else);

const ROOT: &str = "tests/optimizer/";
/// Optimize a given SQF file and return the optimized statements.
fn optimize(file: &str) -> Statements {
    process(file).2.optimize()
}
fn process(file: &str) -> (WorkspacePath, Processed, Statements) {
    let folder = std::path::PathBuf::from(ROOT);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join(format!("{file}.sqf")).unwrap();
    let processed = Processor::run(
        &source,
        &hemtt_common::config::PreprocessorOptions::default(),
    )
    .unwrap();
    let mut sqf = hemtt_sqf::parser::run(&Database::a3(false), &processed).unwrap();
    sqf.testing_clear_issues();
    (source, processed, sqf)
}

#[test]
#[ignore = "helper to do literal A/B sqfc testing"]
/// # Panics
pub fn dev_testing() {
    let (source, processed, statements) = process("dev");
    let file_a = source.with_extension("a.sqfc").unwrap();
    let mut out = file_a.create_file().unwrap();
    statements.compile_to_writer(&processed, &mut out).unwrap();
    let statements = statements.optimize();
    let file_b = source.with_extension("b.sqfc").unwrap();
    let mut out = file_b.create_file().unwrap();
    statements.compile_to_writer(&processed, &mut out).unwrap();
}
