use std::fs::File;

use hemtt_pbo::ReadablePbo;
use hemtt_signing::BIPrivateKey;

const ROOT: &str = "tests/bootstrap/";

#[test]
fn bootstrap() {
    for file in std::fs::read_dir(ROOT).unwrap() {
        let file = file.unwrap();

        // If we write the private key, does it match the original?
        let private =
            BIPrivateKey::read(&mut File::open(file.path().join("test.biprivatekey")).unwrap())
                .unwrap();
        let mut buffer = Vec::new();
        private.write_danger(&mut buffer).unwrap();
        assert_eq!(
            buffer,
            std::fs::read(file.path().join("test.biprivatekey")).unwrap()
        );

        // If we write the public key, does it match the original?
        let public_disk = std::fs::read(file.path().join("test.bikey")).unwrap();
        let public = private.to_public_key();
        let mut buffer = Vec::new();
        public.write(&mut buffer).unwrap();
        assert_eq!(public_disk, buffer);

        // Do we generate the stored checksum?
        let mut pbo =
            ReadablePbo::from(File::open(file.path().join("source.pbo")).unwrap()).unwrap();
        let checksum = pbo.gen_checksum().unwrap();
        assert_eq!(&checksum, pbo.checksum());

        // If we sign the PBO, does it match the original?
        let signature_disk = std::fs::read(file.path().join("source.pbo.test.bisign")).unwrap();
        let signature = private
            .sign(&mut pbo, hemtt_pbo::BISignVersion::V3)
            .unwrap();
        let mut buffer = Vec::new();
        signature.write(&mut buffer).unwrap();
        assert_eq!(signature_disk, buffer);
    }
}
