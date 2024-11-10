#![allow(clippy::unwrap_used)]

use std::fs::File;

use hemtt_pbo::{tests::pbo, Checksum};

#[test]
fn exported_mission() {
    let checksum = Checksum::from_bytes([
        26, 16, 177, 232, 100, 38, 220, 28, 108, 190, 133, 74, 93, 171, 69, 59, 116, 181, 149, 252,
    ]);
    let _ = pbo(
        File::open("tests/exported_mission.VR.pbo").unwrap(),
        9,
        true,
        0,
        None,
        None,
        checksum,
        checksum,
    );
}
