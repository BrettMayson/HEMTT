use std::fs::File;

use hemtt_pbo::{Header, ReadablePBO};

fn test_pbo(file: File, file_count: usize, version: &str, prefix: &str) -> ReadablePBO<File> {
    let pbo = ReadablePBO::from(file).unwrap();
    for extension in pbo.extensions() {
        println!("Ext: {:?}", extension);
    }
    assert_eq!(pbo.files().len(), file_count);
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

#[test]
fn ace_weather() {
    let pbo = test_pbo(
        File::open("tests/ace_weather.pbo").unwrap(),
        41,
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
}
