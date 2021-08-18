use std::{fs::File, path::Path};

use hemtt_pbo::Timestamp;

#[test]
fn ace_weather_cba6f72c() {
    let mut pbo = hemtt_pbo::test::pbo(
        File::open("tests/ace_weather.pbo_cba6f72c").unwrap(),
        41,
        true,
        3,
        "cba6f72c",
        "z\\ace\\addons\\weather",
        vec![
            210, 213, 255, 98, 5, 201, 111, 118, 217, 52, 219, 91, 163, 179, 230, 89, 98, 139, 31,
            78,
        ],
    );
    hemtt_pbo::test::header(
        pbo.files().first().unwrap(),
        "$PBOPREFIX$.backup",
        0,
        20,
        0,
        Timestamp::from_u32(1543422611),
        20,
    );
    hemtt_pbo::test::header(
        &pbo.header("$PBOPREFIX$.backup").unwrap(),
        "$PBOPREFIX$.backup",
        0,
        20,
        0,
        Timestamp::from_u32(1543422611),
        20,
    );
    hemtt_pbo::test::file(
        &mut pbo,
        "XEH_preStart.sqf",
        "#include \"script_component.hpp\"\r\n\r\n#include \"XEH_PREP.hpp\"\r\n".to_string(),
    );
    hemtt_pbo::test::writeable_pbo(pbo, File::open("tests/ace_weather.pbo_cba6f72c").unwrap());
}

#[test]
fn ace_weather_8bd4922f() {
    let mut pbo = hemtt_pbo::test::pbo(
        File::open("tests/ace_weather.pbo_8bd4922f").unwrap(),
        45,
        false,
        3,
        "8bd4922f",
        "z\\ace\\addons\\weather",
        vec![
            182, 44, 18, 201, 133, 232, 236, 162, 127, 37, 203, 45, 42, 137, 130, 36, 120, 104,
            187, 203,
        ],
    );
    hemtt_pbo::test::header(
        pbo.files().first().unwrap(),
        "$PBOPREFIX$.backup",
        0,
        20,
        0,
        Timestamp::from_u32(1615389445),
        20,
    );
    hemtt_pbo::test::header(
        &pbo.header("$PBOPREFIX$.backup").unwrap(),
        "$PBOPREFIX$.backup",
        0,
        20,
        0,
        Timestamp::from_u32(1615389445),
        20,
    );
    hemtt_pbo::test::file(
        &mut pbo,
        "XEH_preStart.sqf",
        "#include \"script_component.hpp\"\r\n\r\n#include \"XEH_PREP.hpp\"\r\n".to_string(),
    );
}

#[test]
fn bi_3den() {
    if !Path::new("tests/3den.pbo").exists() {
        return;
    }
    let mut pbo = hemtt_pbo::test::pbo(
        File::open("tests/3den.pbo").unwrap(),
        368,
        true,
        3,
        "149197",
        "a3\\3den",
        vec![
            57, 137, 163, 39, 148, 153, 116, 24, 229, 159, 191, 235, 207, 97, 198, 246, 142, 171,
            33, 230,
        ],
    );
    hemtt_pbo::test::header(
        pbo.files().first().unwrap(),
        "config.bin",
        0,
        0,
        0,
        Timestamp::from_u32(1601975345),
        516713,
    );
    hemtt_pbo::test::header(
        &pbo.header("config.bin").unwrap(),
        "config.bin",
        0,
        0,
        0,
        Timestamp::from_u32(1601975345),
        516713,
    );
    // hemtt_pbo::test::file(pbo.retrieve("XEH_preStart.sqf").unwrap(), "#include \"script_component.hpp\"\r\n\r\n#include \"XEH_PREP.hpp\"\r\n".to_string());
    hemtt_pbo::test::writeable_pbo(pbo, File::open("tests/3den.pbo").unwrap());
}
