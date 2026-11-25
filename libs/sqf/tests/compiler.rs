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
    compile_files(name, source, &[])
}

fn compile_files(
    name: &str,
    source: &str,
    extra: &[(String, String)],
) -> Result<Compiled, hemtt_sqf::compiler::CompileError> {
    let folder =
        std::env::temp_dir().join(format!("hemtt_sqf_compile_{name}_{}", std::process::id()));
    fs_err::create_dir_all(&folder).unwrap();
    fs_err::write(folder.join("test.sqf"), source).unwrap();
    for (file, content) in extra {
        fs_err::write(folder.join(file), content).unwrap();
    }
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

fn assignment_lines(compiled: &Compiled) -> Vec<u16> {
    let hemtt_sqf::compiler::serializer::Constant::Code(code) =
        &compiled.constants_cache[usize::from(compiled.entry_point)]
    else {
        panic!("entry point is not code");
    };
    code.contents
        .iter()
        .filter_map(|instruction| match instruction {
            hemtt_sqf::compiler::serializer::Instruction::AssignTo(_, info) => Some(info.file_line),
            _ => None,
        })
        .collect()
}

#[test]
fn line_past_u16_is_an_error() {
    let source = format!("early = 1;\n{}late = 2;\n", "\n".repeat(70_000));
    let result = compile_source("lines", &source);
    assert!(
        matches!(
            result,
            Err(hemtt_sqf::compiler::CompileError::LineTooLarge(line)) if line > 65_535
        ),
        "{:?}",
        result.map(|c| assignment_lines(&c))
    );
}

#[test]
fn line_below_u16_is_kept() {
    let source = format!("early = 1;\n{}late = 2;\n", "\n".repeat(65_000));
    let lines = assignment_lines(&compile_source("lines_ok", &source).unwrap());
    assert_eq!(lines.len(), 2);
    assert!(lines[0] < 2, "early line is kept, got {}", lines[0]);
    assert!(
        (65_000..=65_002).contains(&lines[1]),
        "late line is kept exactly, got {}",
        lines[1]
    );
}

#[test]
fn too_many_files() {
    let extra = (0..260)
        .map(|i| (format!("inc_{i}.sqf"), format!("a{i} = {i};\n")))
        .collect::<Vec<_>>();
    let mut source = String::new();
    for (file, _) in &extra {
        writeln!(source, "#include \"{file}\"").unwrap();
    }
    let result = compile_files("files", &source, &extra);
    assert!(
        matches!(
            result,
            Err(hemtt_sqf::compiler::CompileError::TooManyFiles(n)) if n > 255
        ),
        "{:?}",
        result.map(|c| c.file_names.len())
    );
}
