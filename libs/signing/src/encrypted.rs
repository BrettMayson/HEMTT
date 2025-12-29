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

    // KDF params
    mem_cost_kib: u32,
    iterations: u32,
    parallelism: u32,
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

/// Encrypts the private key bytes and writes them to disk.
pub fn encrypt(data: &[u8], password: &str) -> Result<Vec<u8>, Error> {
    let mem_cost_kib = 64 * 1024;
    let iterations = 5;
    let parallelism = 1;

    let salt: [u8; 16] = rand::random();

    let key = derive_key_from_password(password, &salt, mem_cost_kib, iterations, parallelism)?;

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
        mem_cost_kib,
        iterations,
        parallelism,
    };

    bincode::serialize(&blob).map_err(|e| {
        Error::Io(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e,
        )))
    })
}

/// Reads and decrypts the key file from disk.
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
        blob.mem_cost_kib,
        blob.iterations,
        blob.parallelism,
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
        let start = std::time::Instant::now();
        let encrypted = encrypt(data, password).expect("encryption failed");
        println!("Encryption took {:?}", start.elapsed());
        let start = std::time::Instant::now();
        let decrypted = decrypt(&encrypted, password).expect("decryption failed");
        println!("Decryption took {:?}", start.elapsed());
        assert_eq!(data.to_vec(), decrypted);
    }
}
