#![allow(clippy::unwrap_used)]

use std::{fs::File, path::PathBuf};

use hemtt_pbo::ReadablePbo;
use hemtt_signing::{BIPrivateKey, BIPublicKey, BISign};
use rsa::BoxedUint;

#[test]
fn write() {
    let file = PathBuf::from("tests/ace_ai_3.15.2.69");

    // If we write the private key, does it match the original?
    let private =
        BIPrivateKey::read(&mut File::open(file.join("test.biprivatekey")).unwrap()).unwrap();
    let mut buffer = Vec::new();
    private.write_danger(&mut buffer).unwrap();
    assert_eq!(
        buffer,
        std::fs::read(file.join("test.biprivatekey")).unwrap()
    );

    // If we write the public key, does it match the original?
    let public_disk = std::fs::read(file.join("test.bikey")).unwrap();
    let public = private.to_public_key();
    let mut buffer = Vec::new();
    public.write(&mut buffer).unwrap();
    assert_eq!(public_disk, buffer);

    // Do we generate the stored checksum?
    println!("opening file: {:?}", file.join("source.pbo"));
    println!("cwd: {:?}", std::env::current_dir().unwrap());
    // print tree in cwd
    for entry in std::fs::read_dir("./tests/ace_ai_3.15.2.69").unwrap() {
        let entry = entry.unwrap();
        println!("{:?}", entry.path());
    }
    let mut pbo = ReadablePbo::from(File::open(file.join("source.pbo")).unwrap()).unwrap();
    let checksum = pbo.gen_checksum().unwrap();
    assert_eq!(&checksum, pbo.checksum());

    // If we sign the PBO, does it match the original?
    let signature_disk = std::fs::read(file.join("source.pbo.test.bisign")).unwrap();
    let signature = private
        .sign(&mut pbo, hemtt_pbo::BISignVersion::V3)
        .unwrap();
    let mut buffer = Vec::new();
    signature.write(&mut buffer).unwrap();
    assert_eq!(signature_disk, buffer);
    let signature_disk = BISign::read(&mut &signature_disk[..]).unwrap();
    assert_eq!(signature_disk.modulus(), signature.modulus());
    assert_eq!(signature_disk.exponent(), signature.exponent());
    assert_eq!(signature_disk.authority(), signature.authority());
    assert_eq!(signature_disk.version(), signature.version());
    assert_eq!(signature_disk.length(), signature.length());
    assert_eq!(signature_disk.signatures(), signature.signatures());
    assert_eq!(
        signature_disk.signatures_modpow(),
        signature.signatures_modpow()
    );

    let public_disk = BIPublicKey::read(&mut File::open(file.join("test.bikey")).unwrap()).unwrap();
    public_disk.verify(&mut pbo, &signature).unwrap();
    assert_eq!(public_disk.modulus(), signature.modulus());
    assert_eq!(public_disk.exponent(), signature.exponent());
    assert_eq!(public_disk.authority(), signature.authority());
    assert_eq!(public_disk.length(), signature.length());
}

#[test]
fn read_signature() {
    let file = PathBuf::from("tests/ace_ai_3.15.2.69/source.pbo.test.bisign");
    let bisign = BISign::read(&mut File::open(file).unwrap()).unwrap();
    assert_eq!(bisign.authority(), "test");
    assert_eq!(bisign.version(), hemtt_pbo::BISignVersion::V3);
    assert_eq!(bisign.length(), 1024);
    assert_eq!(bisign.exponent(), &BoxedUint::from_words([65537u64]));
    assert_eq!(
        bisign.modulus(),
        &BoxedUint::from_words([
            3_383_022_893_987_068_657,
            211_522_787_039_626_673,
            12_924_607_435_213_790_771,
            4_642_736_248_734_124_677,
            13_049_545_899_981_164_527,
            5_836_844_033_225_426_751,
            18_151_108_490_666_601_265,
            12_542_211_595_622_881_391,
            9_775_904_686_761_608_895,
            9_316_370_910_833_152_348,
            14_627_999_956_071_527_320,
            12_883_383_326_514_718_719,
            15_374_746_912_982_504_272,
            4_911_298_651_162_881_918,
            2_378_468_947_387_679_438,
            13_201_642_397_579_307_866
        ])
    );
    assert_eq!(
        bisign.modulus_display(2),
        "b735a7d0b7736b5a2102\n  04e7ee5702ce4428701a\n  206b2f7ed55e12dbc3db\n  1350b2caf8e05a19a7ff\n  cb0118752412b398814a\n  65fbaffe115c87aafd84\n  848086bfae0ee2fea6e3\n  186ffbe5b100e9c26b31\n  5100a2b929c2873fb519\n  4cdb48884fef406e5005\n  1ba08a85b35d6dfc1d4a\n  5e3302ef7adaa765c5b1\n  2ef2e965e71cfaf1"
    );
}
