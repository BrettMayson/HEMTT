use std::io::Read;

use hemtt_config::rapify::Rapify;
use hemtt_preprocessor::Processor;

const ROOT: &str = "tests/rapify/";

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
        .memory()
        .finish()
        .unwrap();
    let source = workspace.join("source.hpp").unwrap();
    let processed = Processor::run(&source).unwrap();
    let parsed = hemtt_config::parse(&processed);
    if let Err(e) = &parsed {
        println!("{:#?}", e);
        std::fs::write(folder.join("stderr.ansi"), e.join("\n")).unwrap();
        std::fs::write(folder.join("processed.txt"), processed.as_string()).unwrap();
        panic!("failed to parse")
    };
    let parsed = parsed.unwrap();
    let mut expected = Vec::new();
    std::fs::File::open(folder.join("expected.bin"))
        .unwrap()
        .read_to_end(&mut expected)
        .unwrap();
    let mut output = Vec::new();
    let written = parsed.config().rapify(&mut output, 0).unwrap();
    assert_eq!(written, parsed.config().rapified_length());
    assert_eq!(output, expected);
}

bootstrap!(ace_main);
bootstrap!(cba_multiline);
bootstrap!(delete_class);
bootstrap!(external_class);
bootstrap!(join_digit);
bootstrap!(join_in_ident);
bootstrap!(join);
bootstrap!(nested_array);
bootstrap!(numbers);
bootstrap!(procedural_texture);
bootstrap!(single_class);
