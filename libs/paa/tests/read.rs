#![allow(clippy::unwrap_used)]

use hemtt_paa::PaXType;

#[test]
fn read_dxt1() {
    let file = fs_err::File::open("tests/dxt1.paa").unwrap();
    let paa = hemtt_paa::Paa::read(file).unwrap();
    assert_eq!(paa.format(), &PaXType::DXT1);
    assert_eq!(paa.taggs().len(), 3);
    assert!(paa.taggs().contains_key(&"SFFO".to_string()));
    assert!(paa.taggs().contains_key(&"CGVA".to_string()));
    assert!(paa.taggs().contains_key(&"CXAM".to_string()));
    let mipmap = &paa.maps()[0].0;
    assert_eq!(mipmap.width(), 512);
    assert!(mipmap.is_compressed());
    assert_eq!(mipmap.format(), &PaXType::DXT1);
    assert_eq!(mipmap.data().len(), 4716);
    paa.maps()[0].0.get_image().expect("decodes");
}

#[test]
fn read_dxt5() {
    let file = fs_err::File::open("tests/dxt5.paa").unwrap();
    let paa = hemtt_paa::Paa::read(file).unwrap();
    assert_eq!(paa.format(), &PaXType::DXT5);
    assert_eq!(paa.taggs().len(), 4);
    assert!(paa.taggs().contains_key(&"SFFO".to_string()));
    assert!(paa.taggs().contains_key(&"CGVA".to_string()));
    assert!(paa.taggs().contains_key(&"CXAM".to_string()));
    assert!(paa.taggs().contains_key(&"GALF".to_string()));
    let mipmap = &paa.maps()[0].0;
    assert_eq!(mipmap.width(), 64);
    assert!(!mipmap.is_compressed());
    assert_eq!(mipmap.format(), &PaXType::DXT5);
    assert_eq!(mipmap.data().len(), 4096);
    paa.maps()[0].0.get_image().expect("decodes");
}

#[test]
fn read_argba5() {
    let file = fs_err::File::open("tests/argba5.paa").unwrap();
    let paa = hemtt_paa::Paa::read(file).unwrap();
    assert_eq!(paa.format(), &PaXType::ARGBA5);
    assert_eq!(paa.taggs().len(), 3);
    assert!(paa.taggs().contains_key(&"SFFO".to_string()));
    assert!(paa.taggs().contains_key(&"CGVA".to_string()));
    assert!(paa.taggs().contains_key(&"CXAM".to_string()));
    let mipmap = &paa.maps()[0].0;
    assert_eq!(mipmap.width(), 196);
    assert!(mipmap.is_compressed());
    assert_eq!(mipmap.format(), &PaXType::ARGBA5);
    assert_eq!(mipmap.data().len(), 13719);
    paa.maps()[0].0.get_image().expect("decodes");
}

/// Build a mipmap declaring `width` x `height` with `len` bytes behind it.
fn mipmap(format: PaXType, width: u16, height: u16, len: usize) -> hemtt_paa::MipMap {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&height.to_le_bytes());
    #[allow(clippy::cast_possible_truncation)]
    bytes.extend_from_slice(&[len as u8, (len >> 8) as u8, (len >> 16) as u8]); // u24
    bytes.extend_from_slice(&vec![0; len]);
    hemtt_paa::MipMap::from_stream(format, &mut std::io::Cursor::new(bytes)).expect("parses")
}

/// A malformed PAA can declare the format's maximum dimensions with almost no
/// data behind them. That has to be rejected before it reaches an allocation:
/// the size is ~17GB, and a failed allocation aborts rather than unwinding.
#[test]
fn oversized_dimensions_are_rejected() {
    let err = mipmap(PaXType::ARGB8, u16::MAX, u16::MAX, 8)
        .get_image()
        .expect_err("too large to decode");
    assert!(err.contains("too large"), "{err}");
}

/// The block decoder indexes without bounds checks, so a mipmap declaring more
/// pixels than it has bytes for has to be rejected before reaching it.
#[test]
fn truncated_mipmap_is_rejected() {
    // DXT1 at 64x64 needs 2048 bytes
    let err = mipmap(PaXType::DXT1, 64, 64, 8)
        .get_image()
        .expect_err("truncated");
    assert!(err.contains("truncated"), "{err}");

    // and enough bytes still decodes
    assert!(mipmap(PaXType::DXT1, 64, 64, 2048).get_image().is_ok());
}

/// `PaXType::image_size` and `decompress` do not implement these, so they
/// panic. The parser accepts them, so decoding has to refuse them first.
#[test]
fn unimplemented_formats_are_rejected() {
    for format in [PaXType::DXT2, PaXType::DXT4] {
        let err = mipmap(format, 64, 64, 2048)
            .get_image()
            .expect_err("unsupported");
        assert!(err.contains("unsupported"), "{err}");
    }
}
