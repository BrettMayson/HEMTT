#![allow(clippy::unwrap_used)]

//! `hemtt release` end to end: every PBO it produces must carry a signature
//! that verifies against the key it ships, and `hemtt utils verify` must
//! agree, including failing when it shouldn't verify.

mod common;

use std::path::{Path, PathBuf};

use common::{files_with_extension, fixture, run};
use hemtt_pbo::ReadablePbo;
use hemtt_signing::{BIPublicKey, BISign};
use hemtt_test::directory::TemporaryDirectory;
use sealed_test::prelude::*;

const RELEASE: &str = ".hemttout/release";

/// Release `workspace_simple`, keep the returned directory alive for the test
#[must_use]
fn release() -> TemporaryDirectory {
    let directory = fixture("workspace_simple");
    run(&["release", "--no-archive"]).succeeds();
    directory
}

fn key() -> PathBuf {
    let keys = files_with_extension(format!("{RELEASE}/keys"), "bikey");
    assert_eq!(keys.len(), 1, "one key is shipped: {keys:?}");
    keys.into_iter().next().unwrap()
}

fn signature_for(pbo: &Path, authority: &str) -> PathBuf {
    pbo.with_extension(format!("pbo.{authority}.bisign"))
}

#[sealed_test]
fn every_pbo_is_signed_and_verifies() {
    let _directory = fixture("workspace_simple");
    run(&["release", "--no-archive"]).succeeds();

    let public = BIPublicKey::read(&mut fs_err::File::open(key()).unwrap()).unwrap();
    let pbos = files_with_extension(format!("{RELEASE}/addons"), "pbo");
    assert_eq!(pbos.len(), 2, "both addons are built: {pbos:?}");
    for pbo_path in pbos {
        let mut pbo = ReadablePbo::from(fs_err::File::open(&pbo_path).unwrap()).unwrap();
        assert!(pbo.is_sorted().is_ok(), "{}", pbo_path.display());
        assert_eq!(
            &pbo.gen_checksum().unwrap(),
            pbo.checksum(),
            "{}",
            pbo_path.display()
        );
        assert!(
            pbo.properties().contains_key("prefix"),
            "{}",
            pbo_path.display()
        );

        let signature_path = signature_for(&pbo_path, public.authority());
        let signature = BISign::read(&mut fs_err::File::open(&signature_path).unwrap()).unwrap();
        public
            .verify(&mut pbo, &signature)
            .unwrap_or_else(|e| panic!("{}: {e}", pbo_path.display()));
    }
}

#[sealed_test]
fn utils_verify_accepts_a_release() {
    let _directory = release();
    let key = key();
    for pbo in files_with_extension(format!("{RELEASE}/addons"), "pbo") {
        run(&[
            "utils",
            "verify",
            pbo.to_str().unwrap(),
            key.to_str().unwrap(),
        ])
        .succeeds()
        .prints("Verified!");
    }
}

#[sealed_test]
fn utils_verify_rejects_a_tampered_pbo() {
    let _directory = release();
    let key = key();
    let pbo = files_with_extension(format!("{RELEASE}/addons"), "pbo")
        .into_iter()
        .next()
        .unwrap();
    // Flip a byte in the file data, after the headers
    let mut bytes = fs_err::read(&pbo).unwrap();
    let at = bytes.len() - 30;
    bytes[at] ^= 0xFF;
    fs_err::write(&pbo, bytes).unwrap();

    let result = run(&[
        "utils",
        "verify",
        pbo.to_str().unwrap(),
        key.to_str().unwrap(),
    ]);
    result.fails();
    result.does_not_print("Verified!");
}

#[sealed_test]
fn utils_verify_rejects_another_projects_key() {
    let _directory = release();
    let key = key();
    let pbo = files_with_extension(format!("{RELEASE}/addons"), "pbo")
        .into_iter()
        .next()
        .unwrap();
    let authority = BIPublicKey::read(&mut fs_err::File::open(&key).unwrap())
        .unwrap()
        .authority()
        .to_string();

    // A second release generates a new key with the same authority,
    // its signature must not verify against the first key
    let first_key = fs_err::read(&key).unwrap();
    run(&["release", "--no-archive"]).succeeds();
    fs_err::write(&key, first_key).unwrap();
    assert!(signature_for(&pbo, &authority).exists());

    let result = run(&[
        "utils",
        "verify",
        pbo.to_str().unwrap(),
        key.to_str().unwrap(),
    ]);
    result.fails();
    result.does_not_print("Verified!");
}

#[sealed_test]
fn utils_verify_missing_signature_is_an_error() {
    let _directory = release();
    let key = key();
    let pbo = files_with_extension(format!("{RELEASE}/addons"), "pbo")
        .into_iter()
        .next()
        .unwrap();
    for signature in files_with_extension(format!("{RELEASE}/addons"), "bisign") {
        fs_err::remove_file(signature).unwrap();
    }
    run(&[
        "utils",
        "verify",
        pbo.to_str().unwrap(),
        key.to_str().unwrap(),
    ])
    .fails();
}
