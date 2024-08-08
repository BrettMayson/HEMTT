#![allow(clippy::unwrap_used)]

use std::io::Read;

use hemtt_config::rapify::Rapify;
use hemtt_preprocessor::Processor;
use hemtt_workspace::LayerType;

const ROOT: &str = "tests/rapify/";

macro_rules! bootstrap {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<config_rapify_ $dir>]() {
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
    let processed = Processor::run(&source).unwrap();
    let parsed = hemtt_config::parse(None, &processed);
    let workspacefiles = hemtt_workspace::reporting::WorkspaceFiles::new();
    if let Err(e) = &parsed {
        let e = e
            .iter()
            .map(|e| e.diagnostic().unwrap().to_string(&workspacefiles))
            .collect::<Vec<_>>();
        std::fs::write(folder.join("stderr.ansi"), e.join("\n")).unwrap();
        std::fs::write(folder.join("processed.txt"), processed.as_str()).unwrap();
        panic!("failed to parse")
    };
    let parsed = parsed.unwrap();
    let mut expected = Vec::new();
    let expected_path = folder.join("expected.bin");
    if !expected_path.exists() {
        let mut file = std::fs::File::create(&expected_path).unwrap();
        parsed.config().rapify(&mut file, 0).unwrap();
        panic!("expected file did not exist, created it");
    };
    std::fs::File::open(expected_path)
        .unwrap()
        .read_to_end(&mut expected)
        .unwrap();
    let mut output = Vec::new();
    let written = parsed.config().rapify(&mut output, 0).unwrap();
    assert_eq!(written, parsed.config().rapified_length());
    assert_eq!(output, expected);
    let vanilla_path = folder.join("cfgconvert.bin");
    if vanilla_path.exists() {
        let mut expected = Vec::new();
        let mut file = std::fs::File::open(&vanilla_path).unwrap();
        file.read_to_end(&mut expected).unwrap();
        assert_eq!(output, expected);
    };
}

bootstrap!(ace_main);
bootstrap!(cba_multiline);
bootstrap!(delete_class);
bootstrap!(eval);
bootstrap!(external_class);
bootstrap!(inheritence_array_extend);
bootstrap!(join_digit);
bootstrap!(join_in_ident);
bootstrap!(join);
bootstrap!(nested_array);
bootstrap!(numbers);
bootstrap!(procedural_texture);
bootstrap!(single_class);
