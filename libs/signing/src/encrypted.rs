use argon2::Argon2;
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::Error;

#[derive(Serialize, Deserialize)]
struct EncryptedBlob {
    salt: Vec<u8>,
    nonce: [u8; 12],
    ciphertext: Vec<u8>,
    kdf_params: KDFParams,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KDFParams {
    pub mem_cost_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

impl Default for KDFParams {
    fn default() -> Self {
        Self {
            mem_cost_kib: 64 * 1024,
            iterations: 4,
            parallelism: 1,
        }
    }
}

fn derive_key_from_password(
    password: &str,
    salt: &[u8],
    mem_cost_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<Key, Error> {
    let params = argon2::Params::new(mem_cost_kib, iterations, parallelism, None).map_err(|e| {
        Error::Io(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            e,
        )))
    })?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut output_key_material = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output_key_material)
        .map_err(|e| {
            Error::Io(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                e,
            )))
        })?;

    let key = Key::try_from(&output_key_material[..]).map_err(|_| {
        Error::Io(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid key length",
        )))
    })?;

    output_key_material.zeroize();

    Ok(key)
}

/// Encrypts the data with the given password.
pub fn encrypt(data: &[u8], password: &str, kdf_params: KDFParams) -> Result<Vec<u8>, Error> {
    let salt: [u8; 16] = rand::random();

    let key = derive_key_from_password(
        password,
        &salt,
        kdf_params.mem_cost_kib,
        kdf_params.iterations,
        kdf_params.parallelism,
    )?;

    let cipher = ChaCha20Poly1305::new(&key);
    let nonce = rand::random::<[u8; 12]>();
    let nonce_ref = Nonce::try_from(&nonce[..]).map_err(|_| {
        Error::Io(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid nonce length",
        )))
    })?;

    let ciphertext = cipher.encrypt(&nonce_ref, data).map_err(|e| {
        Error::Io(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e,
        )))
    })?;

    let blob = EncryptedBlob {
        salt: salt.to_vec(),
        nonce,
        ciphertext,
        kdf_params,
    };

    bincode::serialize(&blob).map_err(|e| {
        Error::Io(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e,
        )))
    })
}

/// Decrypts the data with the given password.
pub fn decrypt(data: &[u8], password: &str) -> Result<Vec<u8>, Error> {
    let blob: EncryptedBlob = bincode::deserialize(data).map_err(|e| {
        Error::Io(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e,
        )))
    })?;

    let key = derive_key_from_password(
        password,
        &blob.salt,
        blob.kdf_params.mem_cost_kib,
        blob.kdf_params.iterations,
        blob.kdf_params.parallelism,
    )?;

    let cipher = ChaCha20Poly1305::new(&key);
    let nonce = Nonce::try_from(&blob.nonce[..]).map_err(|_| {
        Error::Io(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid nonce length",
        )))
    })?;

    let plaintext = cipher
        .decrypt(&nonce, blob.ciphertext.as_ref())
        .map_err(|e| {
            Error::Io(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )))
        })?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let password = "testpassword";
        let data = b"this is a test private key";
        let encrypted = encrypt(data, password, KDFParams::default()).expect("encryption failed");
        let decrypted = decrypt(&encrypted, password).expect("decryption failed");
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_decrypt_wrong_password() {
        let password = "testpassword";
        let wrong_password = "wrongpassword";
        let data = b"this is a test private key";
        let encrypted = encrypt(data, password, KDFParams::default()).expect("encryption failed");
        let result = decrypt(&encrypted, wrong_password);
        assert!(result.is_err());
    }
}
