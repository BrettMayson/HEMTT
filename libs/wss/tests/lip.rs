use fs_err::File;
use hemtt_wss::Wss;

#[test]
fn test_lip_complete() {
    let wss = Wss::from_ogg(File::open("tests/lip_complete.ogg").expect("read"))
        .expect("Failed to read WSS from OGG");
    let lip = wss.to_lip().expect("Failed to convert WSS to LIP");
    std::fs::write("tests/lip_complete.lip", lip).expect("Failed to write LIP file");
}
