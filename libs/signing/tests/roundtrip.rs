#![allow(clippy::unwrap_used, clippy::cast_possible_truncation)]

//! Generate keys, sign PBOs built in memory, and verify them.
//!
//! A signature must verify against the PBO it was made for, and fail
//! for any change to that PBO, a different key, or a different authority.

use std::{io::Cursor, sync::LazyLock};

use hemtt_pbo::{BISignVersion, ReadablePbo, WritablePbo};
use hemtt_signing::{BIPrivateKey, BIPublicKey, BISign, Error};

/// Key generation is slow in debug builds, so the keys are shared between tests
static KEY: LazyLock<BIPrivateKey> =
    LazyLock::new(|| BIPrivateKey::generate(1024, "test_authority").unwrap());
static OTHER_KEY: LazyLock<BIPrivateKey> =
    LazyLock::new(|| BIPrivateKey::generate(1024, "test_authority").unwrap());

const FILES: &[(&str, &[u8])] = &[
    ("config.bin", b"rapified config"),
    ("functions\\fnc_a.sqf", b"systemChat 'a';"),
    ("functions\\fnc_b.sqfc", b"compiled"),
    ("data\\icon.paa", b"paa data"),
    ("sounds\\shot.ogg", b"ogg data"),
];

fn pbo_bytes(files: &[(&str, &[u8])], prefix: &str) -> Vec<u8> {
    let mut pbo = WritablePbo::new();
    for (name, data) in files {
        pbo.add_file(*name, Cursor::new(data.to_vec())).unwrap();
    }
    pbo.add_property("prefix", prefix);
    pbo.add_property("version", "1.0.0");
    let mut out = Vec::new();
    pbo.write(&mut out, true).unwrap();
    out
}

fn readable(bytes: Vec<u8>) -> ReadablePbo<Cursor<Vec<u8>>> {
    ReadablePbo::from(Cursor::new(bytes)).unwrap()
}

fn sign(files: &[(&str, &[u8])], version: BISignVersion) -> BISign {
    KEY.sign(
        &mut readable(pbo_bytes(files, "z\\test\\addons\\main")),
        version,
    )
    .unwrap()
}

fn verify(files: &[(&str, &[u8])], prefix: &str, signature: &BISign) -> Result<(), Error> {
    KEY.to_public_key()
        .verify(&mut readable(pbo_bytes(files, prefix)), signature)
}

#[test]
fn sign_and_verify() {
    for version in [BISignVersion::V2, BISignVersion::V3] {
        let signature = sign(FILES, version);
        assert_eq!(signature.version(), version);
        assert_eq!(signature.authority(), "test_authority");
        assert_eq!(signature.length(), 1024);
        verify(FILES, "z\\test\\addons\\main", &signature).unwrap();
    }
}

#[test]
fn signatures_are_deterministic() {
    let a = sign(FILES, BISignVersion::V3);
    let b = sign(FILES, BISignVersion::V3);
    assert_eq!(a.signatures(), b.signatures());
}

#[test]
fn changed_signed_file_fails() {
    let signature = sign(FILES, BISignVersion::V3);
    let mut changed = FILES.to_vec();
    changed[1] = ("functions\\fnc_a.sqf", b"systemChat 'b';");
    assert!(matches!(
        verify(&changed, "z\\test\\addons\\main", &signature),
        Err(Error::HashMismatch { .. })
    ));
}

#[test]
fn added_file_fails() {
    let signature = sign(FILES, BISignVersion::V3);
    let mut changed = FILES.to_vec();
    changed.push(("extra.sqf", b"1"));
    assert!(matches!(
        verify(&changed, "z\\test\\addons\\main", &signature),
        Err(Error::HashMismatch { .. })
    ));
}

#[test]
fn removed_file_fails() {
    let signature = sign(FILES, BISignVersion::V3);
    assert!(matches!(
        verify(&FILES[1..], "z\\test\\addons\\main", &signature),
        Err(Error::HashMismatch { .. })
    ));
}

#[test]
fn changed_prefix_fails() {
    let signature = sign(FILES, BISignVersion::V3);
    assert!(matches!(
        verify(FILES, "z\\other\\addons\\main", &signature),
        Err(Error::HashMismatch { .. })
    ));
}

#[test]
fn every_file_is_covered_by_the_signature() {
    // V3 does not hash paa content in its third hash, but the first hash
    // is the checksum of the whole PBO, so any change is still caught
    let signature = sign(FILES, BISignVersion::V3);
    let mut changed = FILES.to_vec();
    changed[3] = ("data\\icon.paa", b"different paa data");
    assert!(matches!(
        verify(&changed, "z\\test\\addons\\main", &signature),
        Err(Error::HashMismatch { .. })
    ));
}

#[test]
/// The signature carries the signer's modulus, verifying must not trust it
fn wrong_key_fails() {
    let signature = sign(FILES, BISignVersion::V3);
    let result = OTHER_KEY.to_public_key().verify(
        &mut readable(pbo_bytes(FILES, "z\\test\\addons\\main")),
        &signature,
    );
    assert!(matches!(result, Err(Error::HashMismatch { .. })));
}

#[test]
fn wrong_authority_fails() {
    let signature = sign(FILES, BISignVersion::V3);
    let mut key = Vec::new();
    KEY.to_public_key().write(&mut key).unwrap();
    // The authority is the first, null terminated string in the key
    let renamed = [
        b"other_authority".as_slice(),
        &key[key.iter().position(|b| *b == 0).unwrap()..],
    ]
    .concat();
    let public = BIPublicKey::read(&mut Cursor::new(renamed)).unwrap();
    assert_eq!(public.authority(), "other_authority");
    assert!(matches!(
        public.verify(
            &mut readable(pbo_bytes(FILES, "z\\test\\addons\\main")),
            &signature
        ),
        Err(Error::AuthorityMismatch { .. })
    ));
}

#[test]
fn unsorted_pbo_fails() {
    let signature = sign(FILES, BISignVersion::V3);
    // The writer always sorts, so build an unsorted PBO by hand by swapping
    // the order of two equal length headers
    let bytes = pbo_bytes(&[("b.sqf", b"1"), ("a.sqf", b"1")], "z\\x");
    let mut swapped = bytes.clone();
    let a = bytes.windows(6).position(|w| w == b"a.sqf\0").unwrap();
    let b = bytes.windows(6).position(|w| w == b"b.sqf\0").unwrap();
    swapped[a] = b'b';
    swapped[b] = b'a';
    let mut pbo = readable(swapped);
    assert!(pbo.is_sorted().is_err());
    assert!(matches!(
        KEY.to_public_key().verify(&mut pbo, &signature),
        Err(Error::InvalidFileSorting)
    ));
}

#[test]
fn signature_roundtrip() {
    let signature = sign(FILES, BISignVersion::V3);
    let mut bytes = Vec::new();
    signature.write(&mut bytes).unwrap();
    let read = BISign::read(&mut Cursor::new(&bytes)).unwrap();
    assert_eq!(read.authority(), signature.authority());
    assert_eq!(read.version(), signature.version());
    assert_eq!(read.length(), signature.length());
    assert_eq!(read.signatures(), signature.signatures());
    verify(FILES, "z\\test\\addons\\main", &read).unwrap();

    let mut again = Vec::new();
    read.write(&mut again).unwrap();
    assert_eq!(bytes, again, "writing is stable");
}

#[test]
fn key_roundtrip() {
    let mut private = Vec::new();
    KEY.write_danger(&mut private).unwrap();
    let read = BIPrivateKey::read(&mut Cursor::new(&private)).unwrap();
    assert_eq!(read.authority(), KEY.authority());
    assert_eq!(read.length(), KEY.length());

    // The key read back makes the same signatures
    let signature = read
        .sign(
            &mut readable(pbo_bytes(FILES, "z\\test\\addons\\main")),
            BISignVersion::V3,
        )
        .unwrap();
    assert_eq!(
        signature.signatures(),
        sign(FILES, BISignVersion::V3).signatures()
    );

    let mut public = Vec::new();
    KEY.to_public_key().write(&mut public).unwrap();
    let public = BIPublicKey::read(&mut Cursor::new(&public)).unwrap();
    assert_eq!(public.authority(), KEY.authority());
    assert_eq!(public.modulus(), KEY.to_public_key().modulus());
    public
        .verify(
            &mut readable(pbo_bytes(FILES, "z\\test\\addons\\main")),
            &signature,
        )
        .unwrap();
}

#[test]
fn truncated_files_are_errors() {
    let mut private = Vec::new();
    KEY.write_danger(&mut private).unwrap();
    let mut public = Vec::new();
    KEY.to_public_key().write(&mut public).unwrap();
    let mut signature = Vec::new();
    sign(FILES, BISignVersion::V3)
        .write(&mut signature)
        .unwrap();

    // Skip the authority, so the cut always lands in the binary data
    let start = "test_authority".len() + 1;
    for len in (start..private.len()).step_by(7) {
        assert!(
            BIPrivateKey::read(&mut Cursor::new(&private[..len])).is_err(),
            "private key at {len}"
        );
    }
    for len in (start..public.len()).step_by(3) {
        assert!(
            BIPublicKey::read(&mut Cursor::new(&public[..len])).is_err(),
            "public key at {len}"
        );
    }
    for len in (start..signature.len()).step_by(7) {
        assert!(
            BISign::read(&mut Cursor::new(&signature[..len])).is_err(),
            "signature at {len}"
        );
    }
}

#[test]
/// `hemtt utils inspect` tries each reader to guess a file's type,
/// so reading any other file must be an error, not a panic
fn readers_reject_other_files_without_panicking() {
    let mut inputs: Vec<Vec<u8>> = vec![
        Vec::new(),
        b"class CfgPatches {};".to_vec(),
        // a rapified config header
        b"\0raP\0\0\0\0\x08\0\0\0".repeat(20),
        // authority followed by a length that doesn't match
        [b"auth\0".as_slice(), &[0xFF; 64]].concat(),
        // a valid header claiming a key of 4 billion bits
        {
            let mut v = b"auth\0".to_vec();
            v.extend((u32::MAX / 8 + 20).to_le_bytes());
            v.extend([0; 12]);
            v.extend(u32::MAX.to_le_bytes());
            v.extend(65537u32.to_le_bytes());
            v
        },
    ];
    // Deterministic pseudo random data
    let mut state = 0x1234_5678_u64;
    for len in [8, 32, 100, 500, 2000] {
        inputs.push(
            (0..len)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    state as u8
                })
                .collect(),
        );
    }
    for input in inputs {
        assert!(BISign::read(&mut Cursor::new(&input)).is_err());
        assert!(BIPublicKey::read(&mut Cursor::new(&input)).is_err());
        assert!(BIPrivateKey::read(&mut Cursor::new(&input)).is_err());
    }
}

#[test]
fn even_modulus_is_rejected() {
    let mut public = Vec::new();
    KEY.to_public_key().write(&mut public).unwrap();
    // The modulus starts after the authority, five u32s and the exponent,
    // it is little endian so its first byte decides odd or even
    let modulus = "test_authority".len() + 1 + 4 * 6;
    public[modulus] &= 0xFE;
    assert!(matches!(
        BIPublicKey::read(&mut Cursor::new(&public)),
        Err(Error::InvalidModulus)
    ));
}
