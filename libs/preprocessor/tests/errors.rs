#![allow(clippy::unwrap_used)]

use std::io::Read;

use hemtt_preprocessor::Processor;
use hemtt_workspace::{LayerType, reporting::WorkspaceFiles};

const ROOT: &str = "tests/errors/";

macro_rules! bootstrap {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<pre_error_ $dir>]() {
                check(stringify!($dir));
            }
        }
    };
}

fn check(dir: &str) {
    let folder = std::path::PathBuf::from(ROOT).join(dir);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join("source.hpp").unwrap();
    let processed = Processor::run(&source);
    match processed {
        Ok(config) => {
            panic!("`{:?}` should have failed: {:#?}", folder, config.as_str())
        }
        Err(e) => {
            let mut expected = Vec::new();
            std::fs::File::open(folder.join("stderr.ansi"))
                .unwrap()
                .read_to_end(&mut expected)
                .unwrap();
            let error =
                e.1.get_code()
                    .unwrap()
                    .diagnostic()
                    .unwrap()
                    .to_string(&WorkspaceFiles::new());
            if expected.is_empty() {
                std::fs::write(folder.join("stderr.ansi"), error.replace('\r', "")).unwrap();
            }
            assert_eq!(
                error.replace('\r', ""),
                String::from_utf8(expected).unwrap().replace('\r', "")
            );
        }
    }
}

bootstrap!(pe1_unexpected_token);
bootstrap!(pe2_unexpected_eof);
bootstrap!(pe3_expected_ident);
bootstrap!(pe4_unknown_directive);
bootstrap!(pe5_define_multitoken_argument);
bootstrap!(pe6_change_builtin);
bootstrap!(pe7_if_unit_or_function);
bootstrap!(pe8_if_undefined);
bootstrap!(pe9_function_call_argument_count);
bootstrap!(pe10_function_as_value);
bootstrap!(pe11_expected_function_or_value);
bootstrap!(pe12_include_not_found);
bootstrap!(pe13_include_not_encased);
bootstrap!(pe14_include_unexpected_suffix);
bootstrap!(pe15_if_invalid_operator);
bootstrap!(pe16_if_incompatible_types);
bootstrap!(pe17_double_else);
bootstrap!(pe18_eoi_ifstate);
bootstrap!(pe19_pragma_unknown);
bootstrap!(pe20_pragma_invalid_scope);
bootstrap!(pe21_pragma_invalid_suppress);
bootstrap!(pe22_pragma_invalid_flag);
bootstrap!(pe23_if_has_include);
bootstrap!(pe24_parsing_failed);
bootstrap!(pe25_exec);
bootstrap!(pe26_unsupported_builtin);
