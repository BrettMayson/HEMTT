use hemtt_format::{FormatterConfig, format_sqf};

const ROOT: &str = "tests/sqf/";

macro_rules! sqf {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<sqf_ $dir>]() {
                insta::assert_snapshot!(sqf(stringify!($dir)));
            }
        }
    };
}

sqf!(arrays);
sqf!(blocks);
sqf!(cba);
sqf!(comments);
sqf!(format);
sqf!(hash);
sqf!(if_and);
sqf!(if_not);
sqf!(if);
sqf!(include);
sqf!(macro_path);
sqf!(macros);
sqf!(numbers);
sqf!(preserve_lines);
sqf!(private);
sqf!(sameline);

fn sqf(file: &str) -> String {
    format_sqf(
        &std::fs::read_to_string(format!("{ROOT}{file}.sqf")).expect("Failed to read test file"),
        &FormatterConfig::default(),
    )
    .unwrap_or_else(|err| format!("ERROR: {err}"))
}
