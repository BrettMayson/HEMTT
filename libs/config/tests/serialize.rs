#![allow(clippy::unwrap_used)]

#[cfg(feature = "serde")]
const ROOT: &str = "tests/serialize/";

macro_rules! serialize {
    ($dir:ident) => {
        paste::paste! {
            #[cfg(feature = "serde")]
            #[test]
            fn [<config_serialize_ $dir>]() {
                insta::assert_snapshot!(serialize(stringify!($dir)));
            }
        }
    };
}

serialize!(hello_world);
serialize!(parent);

#[cfg(feature = "serde")]
fn serialize(file: &str) -> String {
    use hemtt_preprocessor::Processor;
    use hemtt_workspace::LayerType;
    let folder = std::path::PathBuf::from(ROOT);
    let workspace = hemtt_workspace::Workspace::builder()
        .physical(&folder, LayerType::Source)
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .unwrap();
    let source = workspace.join(format!("{file}.hpp")).unwrap();
    let processed = Processor::run(
        &source,
        &hemtt_common::config::PreprocessorOptions::default(),
    )
    .unwrap();
    let parsed = hemtt_config::parse(None, &processed).unwrap();
    serde_json::to_string(parsed.config()).unwrap()
}
