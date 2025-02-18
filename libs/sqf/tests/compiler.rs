#![allow(clippy::unwrap_used)]

pub use float_ord::FloatOrd as Scalar;
use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::database::Database;
use hemtt_workspace::LayerType;

macro_rules! compile {
    ($dir: expr, $file:ident) => {
        paste::paste! {
            #[test]
            fn [<simple_ $file>]() {
                let bin = optimize($dir, stringify!($file));
                // bin to hex
                let hex = bin.iter().map(|b| format!("{:02x}", b)).collect::<String>();
                insta::assert_snapshot!(hex);
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

fn optimize(folder: &str, file: &str) -> Vec<u8> {
    let folder = std::path::PathBuf::from(ROOT).join(folder);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join(format!("{file}.sqf")).unwrap();
    let processed = Processor::run(&source).unwrap();
    let mut writer = Vec::new();
    hemtt_sqf::parser::run(&Database::a3(false), &processed)
        .unwrap()
        .optimize()
        .compile_to_writer(&processed, &mut writer)
        .unwrap();
    writer
}
