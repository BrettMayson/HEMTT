use std::io::Read;

use hemtt_preprocessor::Processor;

const ROOT: &str = "tests/warnings/";

macro_rules! bootstrap {
    ($dir:ident) => {
        paste::paste! {
            #[test]
            fn [<pre_warning_ $dir>]() {
                check(stringify!($dir));
            }
        }
    };
}

fn check(dir: &str) {
    let folder = std::path::PathBuf::from(ROOT).join(dir);
    let workspace = hemtt_common::workspace::Workspace::builder()
        .physical(&folder)
        .finish(None)
        .unwrap();
    let source = workspace.join("source.hpp").unwrap();
    let processed = Processor::run(&source);
    match processed {
        Ok(config) => {
            let mut expected = Vec::new();
            std::fs::File::open(folder.join("stderr.ansi"))
                .unwrap()
                .read_to_end(&mut expected)
                .unwrap();
            let warning = config.warnings().first().unwrap().report().unwrap();
            if expected.is_empty() {
                std::fs::write(folder.join("stderr.ansi"), warning.replace('\r', "")).unwrap();
            }
            assert_eq!(
                warning.replace('\r', ""),
                String::from_utf8(expected).unwrap().replace('\r', "")
            );
        }
        Err(e) => {
            panic!(
                "`{:?}` should have succeeded: {:#?}",
                folder,
                e.get_code().unwrap().report().unwrap()
            )
        }
    }
}

bootstrap!(pw1_redefine);
bootstrap!(pw3_padded_arg);
