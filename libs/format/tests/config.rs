use hemtt_format::{FormatterConfig, format_config};

const ROOT: &str = "tests/config/";

macro_rules! config {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<config_ $dir>]() {
                insta::assert_snapshot!(config(stringify!($dir)));
            }
        }
    };
}

config!(array);
config!(basic);
config!(classes);
config!(comments);
config!(eject);
config!(empty);
config!(include);
config!(invalid);
config!(macro);
config!(macro_path);
config!(math);
config!(nested);
config!(numbers);
config!(parent);

fn config(file: &str) -> String {
    format_config(
        &std::fs::read_to_string(format!("{ROOT}{file}.hpp")).expect("Failed to read test file"),
        &FormatterConfig::default(),
    )
    .unwrap_or_else(|err| format!("ERROR: {err}"))
}
