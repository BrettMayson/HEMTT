use std::{fs::File, path::Path};

use hemtt_pbo::{Header, ReadablePbo, Timestamp, WritablePbo};

fn test_pbo(
    file: File,
    file_count: usize,
    sorted: bool,
    extension_count: usize,
    version: &str,
    prefix: &str,
    checksum: Vec<u8>,
) -> ReadablePbo<File> {
    let mut pbo = ReadablePbo::from(file).unwrap();
    assert_eq!(pbo.files().len(), file_count);
    assert_eq!(pbo.extensions().len(), extension_count);
    assert_eq!(pbo.is_sorted().is_ok(), sorted);
    assert_eq!(pbo.extension("version"), Some(&version.to_string()));
    assert_eq!(pbo.extension("prefix"), Some(&prefix.to_string()));
    assert!(pbo.retrieve("not_real").is_none());
    assert!(pbo.header("not_real").is_none());
    if sorted {
        assert_eq!(pbo.checksum(), checksum);
    }
    pbo
}

fn test_writeable_pbo(pbo: ReadablePbo<File>, file: File) {
    let mut writeable: WritablePbo<std::io::Cursor<Box<[u8]>>> = pbo.into();
    let original = ReadablePbo::from(file).unwrap();

    assert_eq!(original.files(), writeable.files_sorted().unwrap());
    assert_eq!(original.extensions(), writeable.extensions());
    assert_eq!(original.checksum(), writeable.checksum().unwrap());
}

fn test_header(
    header: &Header,
    filename: &str,
    method: u32,
    original: u32,
    reserved: u32,
    timestamp: Timestamp,
    size: u32,
) {
    assert_eq!(header.filename(), filename);
    assert_eq!(header.method(), method);
    assert_eq!(header.original(), original);
    assert_eq!(header.reserved(), reserved);
    assert_eq!(header.timestamp(), timestamp);
    assert_eq!(header.size(), size);
}

fn test_file(pbo: &mut ReadablePbo<File>, file: &str, content: String) {
    let data = pbo.retrieve(file).unwrap();
    let data = String::from_utf8(data.into_inner().to_vec()).unwrap();
    assert_eq!(data, content);
    assert_eq!(pbo.header(file).unwrap().size() as usize, data.len());
}

#[test]
fn ace_weather_cba6f72c() {
    let mut pbo = test_pbo(
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
    test_header(
        pbo.files().first().unwrap(),
        "$PBOPREFIX$.backup",
        0,
        20,
        0,
        Timestamp::from_u32(1543422611),
        20,
    );
    test_header(
        &pbo.header("$PBOPREFIX$.backup").unwrap(),
        "$PBOPREFIX$.backup",
        0,
        20,
        0,
        Timestamp::from_u32(1543422611),
        20,
    );
    test_file(
        &mut pbo,
        "XEH_preStart.sqf",
        "#include \"script_component.hpp\"\r\n\r\n#include \"XEH_PREP.hpp\"\r\n".to_string(),
    );
    test_writeable_pbo(pbo, File::open("tests/ace_weather.pbo_cba6f72c").unwrap());
}

#[test]
fn ace_weather_8bd4922f() {
    let mut pbo = test_pbo(
        File::open("tests/ace_weather.pbo_8bd4922f").unwrap(),
        45,
        false,
        3,
        "8bd4922f",
        "z\\ace\\addons\\weather",
        vec![],
    );
    test_header(
        pbo.files().first().unwrap(),
        "$PBOPREFIX$.backup",
        0,
        20,
        0,
        Timestamp::from_u32(1615389445),
        20,
    );
    test_header(
        &pbo.header("$PBOPREFIX$.backup").unwrap(),
        "$PBOPREFIX$.backup",
        0,
        20,
        0,
        Timestamp::from_u32(1615389445),
        20,
    );
    test_file(
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
    let mut pbo = test_pbo(
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
    test_header(
        pbo.files().first().unwrap(),
        "config.bin",
        0,
        0,
        0,
        Timestamp::from_u32(1601975345),
        516713,
    );
    test_header(
        &pbo.header("config.bin").unwrap(),
        "config.bin",
        0,
        0,
        0,
        Timestamp::from_u32(1601975345),
        516713,
    );
    // test_file(pbo.retrieve("XEH_preStart.sqf").unwrap(), "#include \"script_component.hpp\"\r\n\r\n#include \"XEH_PREP.hpp\"\r\n".to_string());
    test_writeable_pbo(pbo, File::open("tests/3den.pbo").unwrap());
}
