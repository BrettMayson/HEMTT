use hemtt_pbo::WritablePbo;

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
    let mut new_pbo = WritablePbo::new();
    let mut new_files = std::collections::HashMap::new();
    for file in pbo.files() {
        let mut content = pbo.file(file.filename()).unwrap().unwrap();
        let mut data = Vec::new();
        content.read_to_end(&mut data).unwrap();
        new_files.insert(file.filename().to_string(), data);
    }
    for file in pbo.files() {
        new_pbo
            .add_file_with_header(
                file.clone(),
                std::io::Cursor::new(new_files[file.filename()].as_slice()),
            )
            .unwrap();
    }
    for ext in pbo.properties() {
        new_pbo.add_property(ext.0, ext.1);
    }
    let mut new_pbo_bin = std::io::Cursor::new(Vec::new());
    new_pbo.write(&mut new_pbo_bin, true).unwrap();
    let mut old_pbo_bin = std::io::Cursor::new(Vec::new());
    File::open("tests/ace_weather.pbo_cba6f72c")
        .unwrap()
        .read_to_end(old_pbo_bin.get_mut())
        .unwrap();
    assert_eq!(old_pbo_bin.get_ref(), new_pbo_bin.get_ref());
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
