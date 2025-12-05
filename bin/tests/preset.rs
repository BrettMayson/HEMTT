#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use hemtt::commands::launch::preset;
use hemtt_common::arma::dlc::DLC;

macro_rules! bootstrap {
    ($preset:ident, $mods:expr, $dlc:expr) => {
        paste::paste! {
            #[test]
            fn [<config_rapify_ $preset>]() {
                preset(stringify!($preset), $mods, $dlc);
            }
        }
    };
}

fn preset(name: &str, mods: usize, dlc: &[DLC]) {
    let html = PathBuf::from("tests/presets")
        .join(name)
        .with_extension("html");
    assert!(html.exists(), "Preset not found: {name}");
    let html = fs_err::read_to_string(html).unwrap();
    let (preset_mods, preset_dlc) = preset::read(name, &html);
    assert_eq!(preset_mods.len(), mods);
    assert_eq!(preset_dlc, dlc);
}

bootstrap!(dart, 3, &[]);
bootstrap!(dlc, 3, &[DLC::Spearhead1944]);
