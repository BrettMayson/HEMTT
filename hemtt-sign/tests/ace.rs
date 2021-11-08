use std::fs::File;

use hemtt_pbo::sync::ReadablePbo;
use hemtt_sign::{BIPublicKey, BISign};

#[test]
// Invalid because Mikero tools are not sorting the headers correctly
fn invalid() {
    let pub_key =
        BIPublicKey::read(&mut File::open("./tests/ace_3.13.6.60.bikey").unwrap()).unwrap();
    let mut pbo = ReadablePbo::from(File::open("./tests/ace_weather.pbo").unwrap()).unwrap();
    let sig = BISign::read(
        &mut File::open("./tests/ace_weather.pbo.ace_3.13.6.60-8bd4922f.bisign").unwrap(),
    )
    .unwrap();
    assert!(pub_key.verify(&mut pbo, &sig).is_err());
}
