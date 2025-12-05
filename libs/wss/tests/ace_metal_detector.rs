use hemtt_wss::Wss;

#[test]
fn ace_metal_detector() {
    let wss = Wss::read(
        &mut fs_err::File::open("tests/ace_metal_detector.wss").expect("Failed to open WSS file"),
    )
    .expect("Failed to read WSS file");
    assert_eq!(wss.compression(), &hemtt_wss::Compression::None);
    let wav = wss.to_wav().expect("Failed to convert WSS to WAV");
    fs_err::write("tests/ace_metal_detector.wav", &wav).expect("Failed to write WAV file");
    let wss2 = Wss::from_wav(&wav[..]).expect("Failed to convert WAV to WSS");
    assert_eq!(wss2.channels(), 1);
    assert_eq!(wss2.sample_rate(), 44100);
    assert_eq!(wss2.bits_per_sample(), 16);
}
