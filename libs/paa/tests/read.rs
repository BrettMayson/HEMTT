#![allow(clippy::unwrap_used)]

use hemtt_paa::PaXType;

#[test]
fn read_dxt1() {
    let file = std::fs::File::open("tests/dxt1.paa").unwrap();
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
    let _ = paa.maps()[0].0.get_image();
}

#[test]
fn read_dxt5() {
    let file = std::fs::File::open("tests/dxt5.paa").unwrap();
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
    let _ = paa.maps()[0].0.get_image();
}
