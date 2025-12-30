#[test]
#[cfg(feature = "generate")]
fn baer_to_paa() {
    let baer_image = image::open("tests/baer.png").expect("Failed to open baer.png");
    let paa_image = hemtt_paa::Paa::from_dynamic(&baer_image, hemtt_paa::PaXType::DXT5)
        .expect("Failed to convert image to PAA");
    assert_eq!(paa_image.format(), &hemtt_paa::PaXType::DXT5);
    assert_eq!(paa_image.maps().len(), 6);
    let mipmap = &paa_image.maps()[0].0;
    assert_eq!(mipmap.width(), 128);
    assert!(!mipmap.is_compressed());
    assert_eq!(mipmap.format(), &hemtt_paa::PaXType::DXT5);
    let output_file_path = "tests/baer_converted.paa";
    let mut output_file =
        fs_err::File::create(output_file_path).expect("Failed to create output PAA file");
    paa_image
        .write(&mut output_file)
        .expect("Failed to write PAA file");
    // try reading it back
    let mut input_file =
        fs_err::File::open(output_file_path).expect("Failed to open output PAA file for reading");
    let read_back_paa =
        hemtt_paa::Paa::read(&mut input_file).expect("Failed to read back PAA file");
    assert_eq!(read_back_paa.format(), &hemtt_paa::PaXType::DXT5);
    assert_eq!(read_back_paa.maps().len(), 6);
}

#[test]
#[cfg(feature = "generate")]
fn non_power_of_two() {
    let image = image::open("tests/non_power_of_two.png").expect("Failed to open non_power_of_two.png");
    let paa_result = hemtt_paa::Paa::from_dynamic(&image, hemtt_paa::PaXType::ARGB8);
    // assert!(paa_result.is_err(), "Expected error for non-power-of-two dimensions");
    let output_file_path = "tests/non_power_of_two_converted.paa";
    let output_file = fs_err::File::create(output_file_path);
    paa_result.expect("Failed to convert non-power-of-two image to PAA").write(
        &mut output_file.expect("Failed to create output PAA file"),
    ).expect("Failed to write PAA file");
}
