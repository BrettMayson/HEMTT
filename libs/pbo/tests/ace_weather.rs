#![allow(clippy::unwrap_used)]

use std::{fs::File, io::Read};

use hemtt_pbo::{Checksum, Mime, WritablePbo};

mod utils;
use sha1::{Digest, Sha1};
use utils::*;

#[allow(clippy::too_many_lines)]
#[test]
fn ace_weather_cba6f72c() {
    let checksum = Checksum::from_bytes([
        210, 213, 255, 98, 5, 201, 111, 118, 217, 52, 219, 91, 163, 179, 230, 89, 98, 139, 31, 78,
    ]);
    let mut pbo = pbo(
        File::open("tests/ace_weather.pbo_cba6f72c").unwrap(),
        41,
        true,
        3,
        "cba6f72c",
        "z\\ace\\addons\\weather",
        checksum,
        checksum,
    );
    header(
        pbo.files().first().unwrap(),
        "$PBOPREFIX$.backup",
        &Mime::Blank,
        20,
        0,
        1_543_422_611,
        20,
    );
    header(
        pbo.header("$PBOPREFIX$.backup").unwrap(),
        "$PBOPREFIX$.backup",
        &Mime::Blank,
        20,
        0,
        1_543_422_611,
        20,
    );
    file(
        &mut pbo,
        "XEH_preStart.sqf",
        "#include \"script_component.hpp\"\r\n\r\n#include \"XEH_PREP.hpp\"\r\n",
    );

    {
        let mut pbo_summary = String::from("# Properties\n");
        for ext in pbo.properties() {
            pbo_summary.push_str(&format!("{}: {}\n", ext.0, ext.1));
        }
        pbo_summary.push_str("\n# Files\n");
        for file in pbo.files_sorted() {
            pbo_summary.push_str(&format!("{}\n", file.filename()));
            pbo_summary.push_str(&format!("  mime {}\n", file.mime()));
            pbo_summary.push_str(&format!("  original {}\n", file.original()));
            pbo_summary.push_str(&format!("  reserved {}\n", file.reserved()));
            pbo_summary.push_str(&format!("  timestamp {}\n", file.timestamp()));
            pbo_summary.push_str(&format!("  size {}\n", file.size()));
            pbo_summary.push_str(&format!(
                "  offset {:?}\n",
                pbo.file_offset(file.filename()).unwrap()
            ));
            pbo_summary.push_str(&format!(" hash {}\n", {
                let mut content = pbo.file(file.filename()).unwrap().unwrap();
                let mut data = Vec::new();
                content.read_to_end(&mut data).unwrap();
                let mut hasher = Sha1::new();
                hasher.update(data);
                let result: Checksum = hasher.finalize().to_vec().into();
                result.hex()
            }));
        }
        pbo_summary.push_str("\n# Checksum\n");
        pbo_summary.push_str(checksum.hex().as_str());
        insta::assert_snapshot!(pbo_summary);
    }

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
    let checksum = new_pbo.write(&mut new_pbo_bin, true).unwrap();
    let mut old_pbo_bin = std::io::Cursor::new(Vec::new());
    File::open("tests/ace_weather.pbo_cba6f72c")
        .unwrap()
        .read_to_end(old_pbo_bin.get_mut())
        .unwrap();
    assert_eq!(old_pbo_bin.get_ref(), new_pbo_bin.get_ref());

    {
        let mut pbo_summary = String::from("# Properties\n");
        for ext in new_pbo.properties() {
            pbo_summary.push_str(&format!("{}: {}\n", ext.0, ext.1));
        }
        pbo_summary.push_str("\n# Files\n");
        for file in new_pbo.files_sorted() {
            pbo_summary.push_str(&format!("{}\n", file.filename()));
            pbo_summary.push_str(&format!("  mime {}\n", file.mime()));
            pbo_summary.push_str(&format!("  original {}\n", file.original()));
            pbo_summary.push_str(&format!("  reserved {}\n", file.reserved()));
            pbo_summary.push_str(&format!("  timestamp {}\n", file.timestamp()));
            pbo_summary.push_str(&format!("  size {}\n", file.size()));
            pbo_summary.push_str(&format!(
                "  offset {:?}\n",
                pbo.file_offset(file.filename()).unwrap()
            ));
            pbo_summary.push_str(&format!(" hash {}\n", {
                let t = &new_files[file.filename()];
                let mut hasher = Sha1::new();
                hasher.update(t);
                let result: Checksum = hasher.finalize().to_vec().into();
                result.hex()
            }));
        }
        pbo_summary.push_str("\n# Checksum\n");
        pbo_summary.push_str(checksum.hex().as_str());
        insta::assert_snapshot!(pbo_summary);
    }
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
        Checksum::from_bytes([
            192, 194, 71, 145, 26, 138, 140, 97, 35, 238, 93, 21, 54, 70, 202, 148, 73, 239, 125,
            183,
        ]),
    );
    header(
        pbo.files().first().unwrap(),
        "$PBOPREFIX$.backup",
        &Mime::Blank,
        20,
        0,
        1_615_389_445,
        20,
    );
    header(
        pbo.header("$PBOPREFIX$.backup").unwrap(),
        "$PBOPREFIX$.backup",
        &Mime::Blank,
        20,
        0,
        1_615_389_445,
        20,
    );
    file(
        &mut pbo,
        "XEH_preStart.sqf",
        "#include \"script_component.hpp\"\r\n\r\n#include \"XEH_PREP.hpp\"\r\n",
    );
}
