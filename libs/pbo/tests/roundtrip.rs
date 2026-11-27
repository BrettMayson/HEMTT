#![allow(clippy::unwrap_used)]

//! Write PBOs with [`WritablePbo`], read them back with [`ReadablePbo`],
//! and check that nothing is lost or changed on the way.
//!
//! Also feeds the reader truncated and corrupted input, which must fail
//! with an error instead of panicking or hanging.

use std::io::{Cursor, Read};

use hemtt_pbo::{BISignVersion, Error, Mime, ReadablePbo, WritablePbo};

type Files<'a> = &'a [(&'a str, &'a [u8])];

fn build(files: Files, properties: &[(&str, &str)], with_properties: bool) -> (Vec<u8>, Vec<u8>) {
    let mut pbo = WritablePbo::new();
    for (name, data) in files {
        pbo.add_file(*name, Cursor::new(data.to_vec())).unwrap();
    }
    for (key, value) in properties {
        pbo.add_property(*key, *value);
    }
    let mut out = Vec::new();
    let checksum = pbo.write(&mut out, with_properties).unwrap();
    (out, checksum.as_bytes().to_vec())
}

fn read_all(pbo: &mut ReadablePbo<Cursor<Vec<u8>>>, name: &str) -> Vec<u8> {
    let mut data = Vec::new();
    pbo.file(name)
        .unwrap()
        .expect("file exists")
        .read_to_end(&mut data)
        .unwrap();
    data
}

const FILES: Files = &[
    ("config.cpp", b"class CfgPatches {};"),
    ("functions/fnc_test.sqf", b"params [\"_a\"];\n_a + 1"),
    ("XEH_preInit.sqf", b"#include \"script_component.hpp\"\n"),
    ("data/empty.paa", b""),
    ("a.txt", b"first alphabetically"),
];

#[test]
fn roundtrip_files_and_properties() {
    let (bytes, written_checksum) = build(
        FILES,
        &[("prefix", "z\\test\\addons\\main"), ("version", "1.2.3")],
        true,
    );
    let mut pbo = ReadablePbo::from(Cursor::new(bytes)).unwrap();

    assert_eq!(pbo.files().len(), FILES.len());
    assert_eq!(pbo.properties().len(), 2);
    assert_eq!(
        pbo.properties().get("prefix").unwrap(),
        "z\\test\\addons\\main"
    );
    assert_eq!(pbo.properties().get("version").unwrap(), "1.2.3");
    assert!(pbo.is_sorted().is_ok(), "the writer sorts files");
    assert_eq!(pbo.checksum().as_bytes().to_vec(), written_checksum);
    assert_eq!(&pbo.gen_checksum().unwrap(), pbo.checksum());

    for (name, data) in FILES {
        assert_eq!(&read_all(&mut pbo, name), data, "{name}");
        let header = pbo.header(name).unwrap();
        assert_eq!(header.mime(), &Mime::Blank);
        assert_eq!(header.size() as usize, data.len());
    }
}

#[test]
fn forward_slashes_become_backslashes() {
    let (bytes, _) = build(FILES, &[], true);
    let mut pbo = ReadablePbo::from(Cursor::new(bytes)).unwrap();
    assert!(
        pbo.files()
            .iter()
            .any(|h| h.filename() == "functions\\fnc_test.sqf"),
        "stored with a backslash"
    );
    // Both separators find the file
    assert_eq!(
        read_all(&mut pbo, "functions/fnc_test.sqf"),
        read_all(&mut pbo, "functions\\fnc_test.sqf")
    );
}

#[test]
fn lookups_ignore_case() {
    let (bytes, _) = build(FILES, &[], true);
    let mut pbo = ReadablePbo::from(Cursor::new(bytes)).unwrap();
    assert!(pbo.header("CONFIG.CPP").is_some());
    assert_eq!(read_all(&mut pbo, "Config.Cpp"), b"class CfgPatches {};");
    assert!(pbo.file("missing.sqf").unwrap().is_none());
    assert!(pbo.file_offset("missing.sqf").unwrap().is_none());
}

#[test]
fn file_offsets_point_at_the_data() {
    let (bytes, _) = build(FILES, &[("prefix", "x")], true);
    let pbo = ReadablePbo::from(Cursor::new(bytes.clone())).unwrap();
    for (name, data) in FILES {
        let offset = usize::try_from(pbo.file_offset(name).unwrap().unwrap()).unwrap();
        assert_eq!(&&bytes[offset..offset + data.len()], data, "{name}");
    }
}

#[test]
fn files_are_written_sorted_case_insensitively() {
    let (bytes, _) = build(
        &[
            ("b.sqf", b"b"),
            ("A.sqf", b"a"),
            ("c.sqf", b"c"),
            ("B2.sqf", b"b2"),
        ],
        &[],
        true,
    );
    let pbo = ReadablePbo::from(Cursor::new(bytes)).unwrap();
    let names: Vec<_> = pbo
        .files()
        .iter()
        .map(|h| h.filename().to_string())
        .collect();
    assert_eq!(names, ["A.sqf", "b.sqf", "B2.sqf", "c.sqf"]);
}

#[test]
fn prefix_is_always_the_first_property() {
    let (bytes, _) = build(
        &[("a.sqf", b"a")],
        &[("version", "1"), ("author", "me"), ("prefix", "z\\x")],
        true,
    );
    let pbo = ReadablePbo::from(Cursor::new(bytes)).unwrap();
    let keys: Vec<_> = pbo.properties().keys().cloned().collect();
    assert_eq!(keys, ["prefix", "version", "author"]);
}

#[test]
fn property_values_are_trimmed_of_backslashes() {
    let mut pbo: WritablePbo<Cursor<Vec<u8>>> = WritablePbo::new();
    pbo.add_property("prefix", "\\z\\test\\addons\\main\\");
    assert_eq!(pbo.property("prefix"), Some("z\\test\\addons\\main"));
    assert_eq!(
        pbo.remove_property("prefix").as_deref(),
        Some("z\\test\\addons\\main")
    );
    assert_eq!(pbo.property("prefix"), None);
}

#[test]
fn adding_a_file_twice_replaces_it() {
    let mut pbo = WritablePbo::new();
    assert!(
        pbo.add_file("a.sqf", Cursor::new(b"one".to_vec()))
            .unwrap()
            .is_none()
    );
    assert!(
        pbo.add_file("a.sqf", Cursor::new(b"two".to_vec()))
            .unwrap()
            .is_some()
    );
    let mut out = Vec::new();
    pbo.write(&mut out, true).unwrap();
    let mut read = ReadablePbo::from(Cursor::new(out)).unwrap();
    assert_eq!(read.files().len(), 1);
    assert_eq!(read_all(&mut read, "a.sqf"), b"two");
}

#[test]
fn without_properties() {
    let (bytes, _) = build(FILES, &[("prefix", "ignored")], false);
    let mut pbo = ReadablePbo::from(Cursor::new(bytes)).unwrap();
    assert!(pbo.properties().is_empty());
    assert_eq!(pbo.files().len(), FILES.len());
    assert_eq!(read_all(&mut pbo, "config.cpp"), b"class CfgPatches {};");
}

#[test]
fn empty_pbo() {
    let (bytes, _) = build(&[], &[("prefix", "z\\empty")], true);
    let mut pbo = ReadablePbo::from(Cursor::new(bytes)).unwrap();
    assert!(pbo.files().is_empty());
    assert!(pbo.is_sorted().is_ok());
    assert_eq!(&pbo.gen_checksum().unwrap(), pbo.checksum());
    assert!(matches!(pbo.hash_filenames(), Err(Error::NoFiles)));
}

#[test]
fn large_file_roundtrip() {
    // Over 1MB takes the parallel hashing path in the writer
    let large: Vec<u8> = (0..1_500_000u32).map(|i| (i % 251) as u8).collect();
    let (bytes, _) = build(&[("large.bin", &large), ("small.sqf", b"x")], &[], true);
    let mut pbo = ReadablePbo::from(Cursor::new(bytes)).unwrap();
    assert_eq!(read_all(&mut pbo, "large.bin"), large);
    assert_eq!(&pbo.gen_checksum().unwrap(), pbo.checksum());
}

#[test]
fn checksum_changes_with_content() {
    let (_, a) = build(&[("a.sqf", b"one")], &[], true);
    let (_, b) = build(&[("a.sqf", b"two")], &[], true);
    let (_, c) = build(&[("b.sqf", b"one")], &[], true);
    let (_, a2) = build(&[("a.sqf", b"one")], &[], true);
    assert_ne!(a, b, "content is hashed");
    assert_ne!(a, c, "file names are hashed");
    assert_eq!(a, a2, "the same input gives the same checksum");
}

#[test]
fn hash_filenames_skips_empty_files() {
    let (with_empty, _) = build(&[("a.sqf", b"a"), ("empty.sqf", b"")], &[], true);
    let (without_empty, _) = build(&[("a.sqf", b"a")], &[], true);
    let with_empty = ReadablePbo::from(Cursor::new(with_empty)).unwrap();
    let without_empty = ReadablePbo::from(Cursor::new(without_empty)).unwrap();
    assert_eq!(
        with_empty.hash_filenames().unwrap(),
        without_empty.hash_filenames().unwrap()
    );
}

#[test]
fn hash_files_depends_on_version() {
    let (bytes, _) = build(
        &[
            ("config.bin", b"cfg"),
            ("script.sqf", b"sqf"),
            ("model.p3d", b"p3d"),
        ],
        &[],
        true,
    );
    let mut pbo = ReadablePbo::from(Cursor::new(bytes)).unwrap();
    let v2 = pbo.hash_files(BISignVersion::V2).unwrap();
    let v3 = pbo.hash_files(BISignVersion::V3).unwrap();
    assert_ne!(v2, v3, "V2 and V3 hash different sets of files");
}

#[test]
fn hash_files_with_nothing_to_hash() {
    // A PBO with only files V3 doesn't hash still produces a stable hash
    let (a, _) = build(&[("data.paa", b"one")], &[], true);
    let (b, _) = build(&[("data.paa", b"two")], &[], true);
    let mut a = ReadablePbo::from(Cursor::new(a)).unwrap();
    let mut b = ReadablePbo::from(Cursor::new(b)).unwrap();
    assert_eq!(
        a.hash_files(BISignVersion::V3).unwrap(),
        b.hash_files(BISignVersion::V3).unwrap()
    );
}

#[test]
fn data_after_checksum_is_rejected() {
    let (mut bytes, _) = build(FILES, &[], true);
    bytes.push(0);
    assert!(matches!(
        ReadablePbo::from(Cursor::new(bytes)),
        Err(Error::UnexpectedDataAfterChecksum)
    ));
}

#[test]
fn unknown_mime_is_rejected() {
    let (mut bytes, _) = build(&[("a.sqf", b"a")], &[], false);
    // The first header starts at 0: "a.sqf\0" then the mime u32
    bytes[6..10].copy_from_slice(&0xDEAD_BEEF_u32.to_le_bytes());
    assert!(matches!(
        ReadablePbo::from(Cursor::new(bytes)),
        Err(Error::UnsupportedMime(0xDEAD_BEEF))
    ));
}

#[test]
fn truncated_input_is_an_error() {
    let (bytes, _) = build(FILES, &[("prefix", "z\\test")], true);
    // Every prefix of a valid PBO is invalid, and must fail cleanly
    for len in 0..bytes.len() {
        assert!(
            ReadablePbo::from(Cursor::new(bytes[..len].to_vec())).is_err(),
            "a PBO truncated to {len} of {} bytes was accepted",
            bytes.len()
        );
    }
}

#[test]
fn real_pbos_survive_a_rewrite() {
    for path in [
        "tests/ace_weather.pbo_8bd4922f",
        "tests/ace_weather.pbo_cba6f72c",
        "tests/exported_mission.VR.pbo",
    ] {
        let mut original = ReadablePbo::from(fs_err::File::open(path).unwrap()).unwrap();
        let mut rewritten = WritablePbo::new();
        for (key, value) in original.properties().clone() {
            rewritten.add_property(key, value);
        }
        for header in original.files() {
            let mut data = Vec::new();
            original
                .file(header.filename())
                .unwrap()
                .unwrap()
                .read_to_end(&mut data)
                .unwrap();
            rewritten
                .add_file(header.filename(), Cursor::new(data))
                .unwrap();
        }
        let mut out = Vec::new();
        rewritten.write(&mut out, true).unwrap();
        let mut reread = ReadablePbo::from(Cursor::new(out)).unwrap();

        assert_eq!(reread.files().len(), original.files().len(), "{path}");
        assert_eq!(reread.properties(), original.properties(), "{path}");
        assert_eq!(&reread.gen_checksum().unwrap(), reread.checksum(), "{path}");
        for header in original.files() {
            let mut a = Vec::new();
            original
                .file(header.filename())
                .unwrap()
                .unwrap()
                .read_to_end(&mut a)
                .unwrap();
            assert_eq!(
                read_all(&mut reread, header.filename()),
                a,
                "{path}: {}",
                header.filename()
            );
        }
    }
}

#[test]
fn damaged_compressed_file_is_an_error() {
    let (mut bytes, _) = build(&[("a.sqf", b"not lz77 data")], &[], false);
    // Mark the entry as compressed, claiming it decompresses to 100 bytes
    bytes[6..10].copy_from_slice(&0x4370_7273_u32.to_le_bytes());
    bytes[10..14].copy_from_slice(&100u32.to_le_bytes());
    let mut pbo = ReadablePbo::from(Cursor::new(bytes)).unwrap();
    assert!(pbo.header("a.sqf").unwrap().mime().is_compressed());
    assert!(matches!(
        pbo.file("a.sqf"),
        Err(Error::Decompress { file, .. }) if file == "a.sqf"
    ));
    assert!(
        pbo.file_raw("a.sqf").is_ok(),
        "the raw bytes can still be read"
    );
}

/// Replace the UTF-8 `é` (0xC3 0xA9) with the Windows-1252 `é` (0xE9) and a padding
/// byte, so all offsets stay the same
fn to_windows_1252(mut bytes: Vec<u8>) -> Vec<u8> {
    let pos = bytes.windows(2).position(|w| w == [0xC3, 0xA9]).unwrap();
    bytes[pos] = 0xE9;
    bytes[pos + 1] = b'x';
    bytes
}

#[test]
fn non_utf8_filename_is_an_error() {
    let (bytes, _) = build(&[("caf\u{e9}.sqf", b"a"), ("z.sqf", b"z")], &[], true);
    let result = ReadablePbo::from(Cursor::new(to_windows_1252(bytes)));
    let Err(Error::InvalidName(name)) = result else {
        panic!("expected an invalid name error");
    };
    assert_eq!(
        name, "caf\u{FFFD}x.sqf",
        "the name is shown as best it can be"
    );
}

#[test]
fn non_utf8_property_is_an_error() {
    let (bytes, _) = build(&[("a.sqf", b"a")], &[("author", "Ren\u{e9}")], true);
    assert!(matches!(
        ReadablePbo::from(Cursor::new(to_windows_1252(bytes))),
        Err(Error::InvalidName(_))
    ));
}
