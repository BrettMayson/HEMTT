use std::path::PathBuf;

use hemtt_preprocessor::Processor;
use hemtt_sqf::{analyze::analyze, parser::database::Database};

const ROOT: &str = "tests/analyze/";

macro_rules! analyze {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<simple_ $dir>]() {
                test_analyze(stringify!($dir));
            }
        }
    };
}

fn test_analyze(file: &str) {
    let workspace = hemtt_common::workspace::Workspace::builder()
        .physical(&PathBuf::from(ROOT))
        .finish(None)
        .unwrap();
    let source = workspace.join(format!("{file}.sqf")).unwrap();
    let processed = Processor::run(&source).unwrap();
    let database = Database::default();
    match hemtt_sqf::parser::run(&database, &processed) {
        Ok(sqf) => {
            let (warnings, errors) = analyze(&sqf, None, &processed, None, &database);
            for warning in warnings {
                println!("{}", warning.report().unwrap());
            }
        }
        Err(hemtt_sqf::parser::ParserError::ParsingError(e)) => {
            for error in e {
                println!("{}", error.report().unwrap());
            }
            panic!("failed to parse");
        }
        Err(e) => panic!("{e:?}"),
    };
}

analyze!(find_in_str);
analyze!(if_assign);
analyze!(typename);
analyze!(str_format);
analyze!(select_parse_number);
