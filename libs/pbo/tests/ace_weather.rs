include!("utils.rs");

#[test]
fn ace_weather_cba6f72c() {
    let mut pbo = pbo(
        File::open("tests/ace_weather.pbo_cba6f72c").unwrap(),
        41,
        true,
        3,
        "cba6f72c",
        "z\\ace\\addons\\weather",
        Checksum::from_bytes([
            210, 213, 255, 98, 5, 201, 111, 118, 217, 52, 219, 91, 163, 179, 230, 89, 98, 139, 31,
            78,
        ]),
    );
    header(
        pbo.files().first().unwrap(),
        "$PBOPREFIX$.backup",
        Mime::Blank,
        20,
        0,
        1_543_422_611,
        20,
    );
    header(
        pbo.header("$PBOPREFIX$.backup").unwrap(),
        "$PBOPREFIX$.backup",
        Mime::Blank,
        20,
        0,
        1_543_422_611,
        20,
    );
    file(
        &mut pbo,
        "XEH_preStart.sqf",
        "#include \"script_component.hpp\"\r\n\r\n#include \"XEH_PREP.hpp\"\r\n".to_string(),
    );
    // writeable_pbo(pbo, File::open("tests/ace_weather.pbo_cba6f72c").unwrap());
}

#[test]
fn ace_weather_8bd4922f() {
    let mut pbo = pbo(
        File::open("tests/ace_weather.pbo_8bd4922f").unwrap(),
        45,
        false,
        3,
        "8bd4922f",
        "z\\ace\\addons\\weather",
        Checksum::from_bytes([
            182, 44, 18, 201, 133, 232, 236, 162, 127, 37, 203, 45, 42, 137, 130, 36, 120, 104,
            187, 203,
        ]),
    );
    header(
        pbo.files().first().unwrap(),
        "$PBOPREFIX$.backup",
        Mime::Blank,
        20,
        0,
        1_615_389_445,
        20,
    );
    header(
        pbo.header("$PBOPREFIX$.backup").unwrap(),
        "$PBOPREFIX$.backup",
        Mime::Blank,
        20,
        0,
        1_615_389_445,
        20,
    );
    file(
        &mut pbo,
        "XEH_preStart.sqf",
        "#include \"script_component.hpp\"\r\n\r\n#include \"XEH_PREP.hpp\"\r\n".to_string(),
    );
}
