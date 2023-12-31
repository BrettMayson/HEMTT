use std::path::PathBuf;

use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::database::Database;

const ROOT: &str = "tests/simple/";

macro_rules! simple {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<simple_ $dir>]() {
                simple(stringify!($dir));
            }
        }
    };
}

fn simple(file: &str) {
    let workspace = hemtt_common::workspace::Workspace::builder()
        .physical(&PathBuf::from(ROOT))
        .finish(None)
        .unwrap();
    let source = workspace.join(format!("{file}.sqf")).unwrap();
    let processed = Processor::run(&source).unwrap();
    std::fs::write(format!("tests/simple/{file}.sqfp"), processed.as_str()).unwrap();
    let parsed = match hemtt_sqf::parser::run(&Database::default(), &processed) {
        Ok(sqf) => sqf,
        Err(hemtt_sqf::parser::ParserError::ParsingError(e)) => {
            for error in e {
                println!("{}", error.report().unwrap());
            }
            panic!("failed to parse");
        }
        Err(e) => panic!("{e:?}"),
    };
    assert_ne!(parsed.content().len(), 0);
    let mut buffer = Vec::new();
    parsed.compile_to_writer(&processed, &mut buffer).unwrap();
    std::fs::write(format!("tests/simple/{file}.sqfc"), buffer).unwrap();
    std::fs::write(
        format!("tests/simple/{file}.sqfast"),
        format!("{:#?}", parsed.content()),
    )
    .unwrap();
}

simple!(format_font);
simple!(dev);
simple!(eventhandler);
simple!(foreach);
simple!(get_visibility);
simple!(hash_select);
simple!(hello);
simple!(include);
simple!(oneline);
simple!(semicolons);
