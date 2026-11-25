#![allow(clippy::unwrap_used)]

use std::fmt::Write as _;

pub use float_ord::FloatOrd as Scalar;
use hemtt_preprocessor::Processor;
use hemtt_sqf::{compiler::serializer::Compiled, parser::database::Database};
use hemtt_workspace::LayerType;

macro_rules! compile {
    ($dir: expr, $file:ident) => {
        paste::paste! {
            #[test]
            fn [<simple_ $file>]() {
                let (bin, compiled) = optimize($dir, stringify!($file));
                // bin to hex
                let hex = bin.iter().map(|b| format!("{:02x}", b)).collect::<String>();
                insta::assert_snapshot!(hex);
                insta::assert_debug_snapshot!(compiled);
            }
        }
    };
}

compile!("optimizer", consume_array);
compile!("optimizer", static_math);
compile!("optimizer", scalar);
compile!("optimizer", string_case);
compile!("simple", dev);
compile!("simple", eventhandler);
compile!("simple", foreach);
compile!("simple", format_font);
compile!("simple", get_visibility);
compile!("simple", hash_select);
compile!("simple", hello);
compile!("simple", include);
compile!("simple", oneline);
compile!("simple", semicolons);

const ROOT: &str = "tests/";

fn optimize(folder: &str, file: &str) -> (Vec<u8>, Compiled) {
    let folder = std::path::PathBuf::from(ROOT).join(folder);
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
    let mut writer = Vec::new();
    let statements = hemtt_sqf::parser::run(&Database::a3(false), &processed).unwrap();
    let compiled = statements.compile(&processed).unwrap();
    statements
        .optimize()
        .compile_to_writer(&processed, &mut writer)
        .unwrap();
    (writer, compiled)
}

fn compile_source(name: &str, source: &str) -> Result<Compiled, hemtt_sqf::compiler::CompileError> {
    let folder =
        std::env::temp_dir().join(format!("hemtt_sqf_compile_{name}_{}", std::process::id()));
    fs_err::create_dir_all(&folder).unwrap();
    fs_err::write(folder.join("test.sqf"), source).unwrap();
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let processed = Processor::run(
        &workspace.join("test.sqf").unwrap(),
        &hemtt_common::config::PreprocessorOptions::default(),
    )
    .unwrap();
    let statements = hemtt_sqf::parser::run(&Database::a3(false), &processed).unwrap();
    let result = statements.compile(&processed);
    fs_err::remove_dir_all(&folder).unwrap();
    result
}

#[test]
fn too_many_constants() {
    // 70 arrays of 1000 unique numbers, each array is below the length limit
    let mut source = String::new();
    for chunk in 0..70 {
        let items = (0..1000)
            .map(|i| (chunk * 1000 + i).to_string())
            .collect::<Vec<_>>()
            .join(",");
        writeln!(source, "a{chunk} = [{items}];").unwrap();
    }
    assert!(matches!(
        compile_source("constants", &source),
        Err(hemtt_sqf::compiler::CompileError::TooManyConstants)
    ));
}

#[test]
fn constant_array_too_long() {
    let items = vec!["1"; 70_000].join(",");
    assert!(matches!(
        compile_source("array", &format!("a = [{items}];")),
        Err(hemtt_sqf::compiler::CompileError::ListTooLong(70_000))
    ));
}
