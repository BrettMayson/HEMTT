#![allow(clippy::unwrap_used)]

use std::{fs::File, io::Read};

use hemtt_pbo::{Checksum, Header, Mime, ReadablePbo};

#[must_use]
/// # Panics
/// Will panic if there is an issue with the test
pub fn pbo(
    file: File,
    file_count: usize,
    sorted: bool,
    property_count: usize,
    version: &str,
    prefix: &str,
    checksum: Checksum,
) -> ReadablePbo<File> {
    let mut pbo = ReadablePbo::from(file).unwrap();
    assert_eq!(pbo.files().len(), file_count);
    assert_eq!(pbo.properties().len(), property_count);
    assert_eq!(pbo.is_sorted().is_ok(), sorted);
    assert_eq!(pbo.properties().get("version"), Some(&version.to_string()));
    assert_eq!(pbo.properties().get("prefix"), Some(&prefix.to_string()));
    assert!(pbo.file("not_real").unwrap().is_none());
    assert!(pbo.header("not_real").is_none());
    if sorted {
        assert_eq!(pbo.checksum(), &checksum);
        assert_eq!(pbo.gen_checksum().unwrap(), checksum);
    } else {
        // assert_eq!(pbo.gen_checksum().unwrap(), checksum);
    }
    pbo
}

// pub fn writeable_pbo(pbo: ReadablePbo<File>, file: File) {
//     let mut writeable: WritablePbo<std::io::Cursor<Vec<u8>>> = pbo.try_into().unwrap();
//     let original = ReadablePbo::from(file).unwrap();

//     assert_eq!(original.files(), writeable.files_sorted().unwrap());
//     assert_eq!(original.properties(), writeable.properties());
//     assert_eq!(original.checksum(), writeable.checksum().unwrap());
// }

/// # Panics
/// Will panic if there is an issue with the test
pub fn header(
    header: &Header,
    filename: &str,
    method: &Mime,
    original: u32,
    reserved: u32,
    timestamp: u32,
    size: u32,
) {
    assert_eq!(header.filename(), filename);
    assert_eq!(header.mime(), method);
    assert_eq!(header.original(), original);
    assert_eq!(header.reserved(), reserved);
    assert_eq!(header.timestamp(), timestamp);
    assert_eq!(header.size(), size);
}

/// # Panics
/// Will panic if there is an issue with the test
pub fn file(pbo: &mut ReadablePbo<File>, file: &str, content: &str) {
    let mut cursor = pbo.file(file).unwrap().unwrap();
    let mut data = String::new();
    cursor.read_to_string(&mut data).unwrap();
    assert_eq!(data, content);
    assert_eq!(pbo.header(file).unwrap().size() as usize, data.len());
}
