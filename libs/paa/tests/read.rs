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

/// A malformed PAA can declare the format's maximum dimensions with almost no
/// data behind them. Decoding must fail rather than run.
#[test]
fn oversized_dimensions_are_rejected() {
    use std::io::Cursor;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&u16::MAX.to_le_bytes()); // width
    bytes.extend_from_slice(&u16::MAX.to_le_bytes()); // height
    bytes.extend_from_slice(&[8, 0, 0]); // u24 data length
    bytes.extend_from_slice(&[0; 8]);

    let mipmap =
        hemtt_paa::MipMap::from_stream(PaXType::ARGB8, &mut Cursor::new(bytes)).expect("parses");
    assert!(mipmap.get_image().is_err());
}
