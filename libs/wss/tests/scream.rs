use hemtt_wss::Wss;

#[test]
fn scream() {
    let wss = Wss::from_mp3(
        &mut std::fs::File::open("tests/scream.mp3").expect("Failed to open WSS file"),
    )
    .expect("Failed to read WSS file");
    let wav = wss.to_wav().expect("Failed to convert WSS to WAV");
    std::fs::write("tests/scream.wav", &wav).expect("Failed to write WAV file");
    let ogg = wss.to_ogg().expect("Failed to convert WSS to OGG");
    std::fs::write("tests/scream.ogg", &ogg).expect("Failed to write OGG file");
}
