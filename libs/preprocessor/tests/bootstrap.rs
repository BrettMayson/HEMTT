use hemtt_preprocessor::Processor;

const ROOT: &str = "tests/bootstrap/";

macro_rules! bootstrap {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<bootstrap_ $dir>]() {
                check(stringify!($dir));
            }
        }
    };
}

fn check(dir: &str) {
    let folder = std::path::PathBuf::from(ROOT).join(dir);
    let workspace = hemtt_common::workspace::Workspace::builder()
        .physical(&folder)
        .finish()
        .unwrap();
    let source = workspace.join("source.hpp").unwrap();
    let processed = Processor::run(&source);
    if let Err(e) = processed {
        panic!("{}", e.get_code().unwrap().generate_report().unwrap());
    }
    let processed = processed.unwrap();
    let expected = workspace
        .join("expected.hpp")
        .unwrap()
        .read_to_string()
        .unwrap();
    let processed = processed.as_string();
    std::fs::write(folder.join("generated.hpp"), processed).unwrap();
    assert_eq!(processed, expected.replace('\r', ""));
}

bootstrap!(ace_main);
bootstrap!(cba_is_admin);
bootstrap!(cba_multiline);
bootstrap!(comment_edgecase);
bootstrap!(define_builtin);
bootstrap!(define_function);
bootstrap!(define_function_empty);
bootstrap!(define_function_multiline);
bootstrap!(define_inside_else);
bootstrap!(define_multi);
bootstrap!(define_nested);
bootstrap!(define_nested_nested);
bootstrap!(define_single);
bootstrap!(define_undef);
bootstrap!(define_use_define);
bootstrap!(define_with_dash);
bootstrap!(if_nested);
bootstrap!(if_operators);
bootstrap!(if_pass);
bootstrap!(if_read);
bootstrap!(if_value);
bootstrap!(ignore_quoted);
bootstrap!(include);
bootstrap!(include_empty);
bootstrap!(join_digit);
bootstrap!(name_collision);
bootstrap!(procedural_texture);
bootstrap!(quote);
bootstrap!(quote_recursive);
bootstrap!(redefine_external);
bootstrap!(self_recursion);
bootstrap!(sqf);
bootstrap!(sqf_select);
bootstrap!(strings);
bootstrap!(utf);
