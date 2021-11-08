mod private_key;
pub use private_key::BIPrivateKey;

mod public_key;
pub use public_key::BIPublicKey;

mod signature;
pub use signature::{BISign, BISignVersion};

use std::io::{Error, Read, Seek, Write};

use hemtt_pbo::sync::ReadablePbo;
use openssl::bn::BigNum;
use openssl::hash::{DigestBytes, Hasher, MessageDigest};

pub fn generate_hashes<I: Seek + Read>(
    pbo: &mut ReadablePbo<I>,
    version: BISignVersion,
    length: u32,
) -> (BigNum, BigNum, BigNum) {
    let checksum = pbo.checksum();
    let hash1 = checksum.as_slice();

    let mut h = Hasher::new(MessageDigest::sha1()).unwrap();
    h.update(hash1).unwrap();
    h.update(&*namehash(pbo)).unwrap();
    if let Some(prefix) = pbo.extensions().get("prefix") {
        h.update(prefix.as_bytes()).unwrap();
        if !prefix.ends_with('\\') {
            h.update(b"\\").unwrap();
        }
    }
    let hash2 = &*h.finish().unwrap();

    h = Hasher::new(MessageDigest::sha1()).unwrap();
    h.update(&*filehash(pbo, version)).unwrap();
    h.update(&*namehash(pbo)).unwrap();
    if let Some(prefix) = pbo.extensions().get("prefix") {
        h.update(prefix.as_bytes()).unwrap();
        if !prefix.ends_with('\\') {
            h.update(b"\\").unwrap();
        }
    }
    let hash3 = &*h.finish().unwrap();

    (
        pad_hash(hash1, (length / 8) as usize),
        pad_hash(hash2, (length / 8) as usize),
        pad_hash(hash3, (length / 8) as usize),
    )
}

pub fn pad_hash(hash: &[u8], size: usize) -> BigNum {
    let mut vec: Vec<u8> = vec![0, 1];
    vec.resize(size - 36, 255);
    vec.extend(b"\x00\x30\x21\x30\x09\x06\x05\x2b");
    vec.extend(b"\x0e\x03\x02\x1a\x05\x00\x04\x14");
    vec.extend(hash);

    BigNum::from_slice(&vec).unwrap()
}

pub fn namehash<I: Seek + Read>(pbo: &mut ReadablePbo<I>) -> DigestBytes {
    let mut h = Hasher::new(MessageDigest::sha1()).unwrap();

    let files = pbo.files();

    for header in &files {
        let data = pbo.retrieve(header.filename()).unwrap();
        if data.get_ref().is_empty() {
            continue;
        }

        h.update(
            header
                .filename()
                .replace("/", "\\")
                .to_lowercase()
                .as_bytes(),
        )
        .unwrap();
    }

    h.finish().unwrap()
}

pub fn filehash<I: Seek + Read>(pbo: &mut ReadablePbo<I>, version: BISignVersion) -> DigestBytes {
    let mut h = Hasher::new(MessageDigest::sha1()).unwrap();
    let mut nothing = true;

    for header in pbo.files().iter() {
        let ext = header.filename().split('.').last().unwrap();

        match version {
            BISignVersion::V2 => {
                if ext == "paa"
                    || ext == "jpg"
                    || ext == "p3d"
                    || ext == "tga"
                    || ext == "rvmat"
                    || ext == "lip"
                    || ext == "ogg"
                    || ext == "wss"
                    || ext == "png"
                    || ext == "rtm"
                    || ext == "pac"
                    || ext == "fxy"
                    || ext == "wrp"
                {
                    continue;
                }
            }
            BISignVersion::V3 => {
                if ext != "sqf"
                    && ext != "inc"
                    && ext != "bikb"
                    && ext != "ext"
                    && ext != "fsm"
                    && ext != "sqm"
                    && ext != "hpp"
                    && ext != "cfg"
                    && ext != "sqs"
                    && ext != "h"
                    && ext != "sqfc"
                {
                    continue;
                }
            }
        }
        let cursor = pbo.retrieve(header.filename()).unwrap();
        h.update((&cursor).get_ref()).unwrap();
        nothing = false;
    }

    match version {
        BISignVersion::V2 => {
            if nothing {
                h.update(b"nothing").unwrap();
            }
        }
        BISignVersion::V3 => {
            if nothing {
                h.update(b"gnihton").unwrap();
            }
        }
    }

    h.finish().unwrap()
}

fn display_hashes(a: BigNum, b: BigNum) -> (String, String) {
    let hex_a = a.to_hex_str().unwrap().to_lowercase();
    let hex_b = b.to_hex_str().unwrap().to_lowercase();

    if hex_a.len() != hex_b.len() || hex_a.len() <= 40 {
        return (hex_a, hex_b);
    }

    let (padding_a, hash_a) = hex_a.split_at(hex_a.len() - 40);
    let (paddingb, hash_b) = hex_b.split_at(hex_b.len() - 40);

    if padding_a == paddingb {
        (hash_a.to_string(), hash_b.to_string())
    } else {
        (hex_a, hex_b)
    }
}

pub fn write_bignum<O: Write>(output: &mut O, bn: &BigNum, size: usize) -> Result<(), Error> {
    let mut vec: Vec<u8> = bn.to_vec();
    vec = vec.iter().rev().cloned().collect();
    vec.resize(size, 0);

    output.write_all(&vec)
}
