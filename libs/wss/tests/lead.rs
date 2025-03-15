// Copyright music, only used for local testing
// use hemtt_wss::Wss;
//
// #[test]
// fn lead() {
//     let wss =
//         Wss::from_ogg(&mut std::fs::File::open("tests/lead.ogg").expect("Failed to open WSS file"))
//             .expect("Failed to read WSS file");
//     let wav = wss.to_wav().expect("Failed to convert WSS to WAV");
//     std::fs::write("tests/lead.wav", &wav).expect("Failed to write WAV file");
//     let ogg = wss.to_ogg().expect("Failed to convert WSS to OGG");
//     std::fs::write("tests/lead2.ogg", &ogg).expect("Failed to write OGG file");
// }
