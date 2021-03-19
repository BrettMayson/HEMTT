#[test]
fn to_jpg_dxt1() {
    let file = std::fs::File::open("tests/dxt1.paa").unwrap();
    let paa = hemtt_paa::PAA::read(file).unwrap();
    paa.maps[0].get_image();
}

#[test]
fn to_jpg_dxt5() {
    let file = std::fs::File::open("tests/dxt5.paa").unwrap();
    let paa = hemtt_paa::PAA::read(file).unwrap();
    paa.maps[0].get_image();
}
