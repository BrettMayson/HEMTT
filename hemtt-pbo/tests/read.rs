use std::fs::File;

use hemtt_pbo::{Header, ReadablePBO, WritablePBO};

fn test_pbo(file: File, file_count: usize, extension_count: usize, version: &str, prefix: &str) -> ReadablePBO<File> {
    let pbo = ReadablePBO::from(file).unwrap();
    assert_eq!(pbo.files().len(), file_count);
    assert_eq!(pbo.extensions().len(), extension_count);
    assert_eq!(pbo.is_sorted(), true);
    assert_eq!(pbo.extension("version"), Some(&version.to_string()));
    assert_eq!(pbo.extension("prefix"), Some(&prefix.to_string()));
    pbo
}

fn test_header(
    header: &Header,
    filename: &str,
    method: u32,
    original: u32,
    reserved: u32,
    timestamp: u32,
    size: u32,
) {
    assert_eq!(header.filename(), filename);
    assert_eq!(header.method(), method);
    assert_eq!(header.original(), original);
    assert_eq!(header.reserved(), reserved);
    assert_eq!(header.timestamp(), timestamp);
    assert_eq!(header.size(), size);
}

fn test_file(
    data: std::io::Cursor<Box<[u8]>>,
    content: String,
) {
    let data = String::from_utf8(data.into_inner().to_vec()).unwrap();
    assert_eq!(data, content);
}

#[test]
fn ace_weather() {
    let mut pbo = test_pbo(
        File::open("tests/ace_weather.pbo").unwrap(),
        41,
        3,
        "cba6f72c",
        "z\\ace\\addons\\weather",
    );
    test_header(
        pbo.files().first().unwrap(),
        "$PBOPREFIX$.backup",
        0,
        20,
        0,
        1543422611,
        20,
    );
    test_header(
        &pbo.header("$PBOPREFIX$.backup").unwrap(),
        "$PBOPREFIX$.backup",
        0,
        20,
        0,
        1543422611,
        20,
    );
    test_file(pbo.retrieve("XEH_preStart.sqf").unwrap(), "#include \"script_component.hpp\"\r\n\r\n#include \"XEH_PREP.hpp\"\r\n".to_string());
    assert_eq!(pbo.retrieve("not_real").is_none(), true);
    assert_eq!(pbo.header("not_real").is_none(), true);
    let _writeable: WritablePBO<std::io::Cursor<Box<[u8]>>> = pbo.into();
}
