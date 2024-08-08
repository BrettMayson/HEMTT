#![allow(clippy::unwrap_used)]

use hemtt_preprocessor::Processor;
use hemtt_workspace::{reporting::WorkspaceFiles, LayerType};

const ROOT: &str = "tests/bootstrap/";

macro_rules! bootstrap {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<pre_ $dir>]() {
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
    if let Err(e) = processed {
        panic!(
            "{}",
            e.1.get_code()
                .unwrap()
                .diagnostic()
                .unwrap()
                .to_string(&WorkspaceFiles::new())
        );
    }
    let processed = processed.unwrap();
    let expected = workspace
        .join("expected.hpp")
        .unwrap()
        .read_to_string()
        .unwrap();
    let processed = processed.as_str().replace('\r', "");
    std::fs::write(folder.join("generated.hpp"), &processed).unwrap();
    assert_eq!(processed, expected.replace('\r', ""));
}

bootstrap!(ace_main);
bootstrap!(addon_in_ifdef);
bootstrap!(cba_is_admin);
bootstrap!(cba_multiline);
bootstrap!(comment_edgecase);
bootstrap!(define_builtin);
bootstrap!(define_function_empty);
bootstrap!(define_function_multiline);
bootstrap!(define_function);
bootstrap!(define_inside_else);
bootstrap!(define_multi);
bootstrap!(define_nested_nested);
bootstrap!(define_nested);
bootstrap!(define_single);
bootstrap!(define_undef);
bootstrap!(define_use_define);
bootstrap!(define_variable_recursion);
bootstrap!(define_with_dash);
bootstrap!(group_unit);
bootstrap!(hashtag_outside_macro);
bootstrap!(if_digits);
bootstrap!(if_nested);
bootstrap!(if_operators);
bootstrap!(if_pass);
bootstrap!(if_read);
bootstrap!(if_value);
bootstrap!(ignore_if_has_include);
bootstrap!(ignore_quoted);
bootstrap!(include_empty);
bootstrap!(include);
bootstrap!(join_digit);
bootstrap!(join_ignore);
bootstrap!(name_collision);
bootstrap!(procedural_texture);
bootstrap!(quote_recursive);
bootstrap!(quote);
bootstrap!(redefine_external);
bootstrap!(self_recursion);
bootstrap!(sqf_select);
bootstrap!(sqf);
bootstrap!(strings);
bootstrap!(utf);
