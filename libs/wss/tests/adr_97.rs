use std::path::PathBuf;

use hemtt_wss::Wss;

#[test]
fn adr_97_tailtrees_byte() {
    test_file("tests/adr_97_tailtrees.wss");
}

#[test]
fn adr_97_silencershot_01_byte() {
    test_file("tests/adr_97_silencershot_01.wss");
}

#[test]
fn adr_97_closeshot_01_byte() {
    test_file("tests/adr_97_closeshot_01.wss");
}

fn test_file(path: &str) {
    let wav_path = PathBuf::from(path).with_extension("wav");
    let wss = Wss::read(&mut std::fs::File::open(path).expect("Failed to open WSS file"))
        .expect("Failed to read WSS file");
    assert_eq!(wss.compression(), &hemtt_wss::Compression::Byte);
    let wav = wss.to_wav().expect("Failed to convert WSS to WAV");
    std::fs::write(&wav_path, &wav).expect("Failed to write WAV file");
    {
        let wss_byte = Wss::from_wav(&wav[..]).expect("Failed to convert WAV to WSS");
        assert_eq!(wss_byte.channels(), 2);
        assert_eq!(wss_byte.sample_rate(), 44100);
        assert_eq!(wss_byte.bits_per_sample(), 16);
        std::fs::write(
            wav_path.with_extension("byte.wav"),
            wss_byte.to_wav().expect("failed to convert to wav"),
        )
        .expect("Failed to write WAV file");
    }
    {
        let wss_byte = Wss::from_wav(&wav[..]).expect("Failed to convert WAV to WSS");
        assert_eq!(wss_byte.channels(), 2);
        assert_eq!(wss_byte.sample_rate(), 44100);
        assert_eq!(wss_byte.bits_per_sample(), 16);
        std::fs::write(
            wav_path.with_extension("byte.ogg"),
            wss_byte.to_ogg().expect("failed to convert to ogg"),
        )
        .expect("Failed to write WAV file");
    }
    {
        let wss_byte = Wss::from_wav(&wav[..]).expect("Failed to convert WAV to WSS");
        assert_eq!(wss_byte.channels(), 2);
        assert_eq!(wss_byte.sample_rate(), 44100);
        assert_eq!(wss_byte.bits_per_sample(), 16);
        std::fs::write(
            wav_path.with_extension("nibble.wav"),
            wss_byte.to_wav().expect("failed to convert to wav"),
        )
        .expect("Failed to write WAV file");
    }
}
