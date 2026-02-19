use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
};

use crate::error::HushError;
use crate::models::EncryptedSecret;

pub fn encrypt(plaintext: &str, key: &[u8; 32]) -> Result<EncryptedSecret, HushError> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);

    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| HushError::EncryptionError)?;

    Ok(EncryptedSecret {
        ciphertext,
        nonce: nonce_bytes.to_vec(),
    })
}

pub fn decrypt(secret: &EncryptedSecret, key: &[u8; 32]) -> Result<String, HushError> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    if secret.nonce.len() != 12 {
        return Err(HushError::DecryptionError);
    }
    let nonce = Nonce::from_slice(&secret.nonce);

    let decrypted_bytes = cipher
        .decrypt(nonce, secret.ciphertext.as_ref())
        .map_err(|_| HushError::DecryptionError)?;

    String::from_utf8(decrypted_bytes).map_err(|_| HushError::DecryptionError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption_roundtrip() {
        let key = [0u8; 32];
        let message = "hush-is-secure-and-mathematically-sound";

        let encrypted = encrypt(message, &key).expect("Encryption failed");
        let decrypted = decrypt(&encrypted, &key).expect("Decryption failed");

        assert_eq!(
            message, decrypted,
            "Decrypted message must match the original"
        );
    }

    #[test]
    fn test_decryption_fails_with_wrong_key() {
        let key_a = [0u8; 32];
        let key_b = [1u8; 32];
        let message = "secret-data";

        let encrypted = encrypt(message, &key_a).unwrap();
        let result = decrypt(&encrypted, &key_b);

        assert!(
            result.is_err(),
            "Decryption should fail when the wrong key is used"
        );
    }

    #[test]
    fn test_decryption_fails_when_ciphertext_is_tampered() {
        let key = [0u8; 32];
        let message = "my-precious-api-key";

        let mut encrypted = encrypt(message, &key).expect("Encryption failed");

        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0xFF;
        }

        let result = decrypt(&encrypted, &key);

        assert!(
            result.is_err(),
            "Decryption should fail if ciphertext is tampered with"
        );
    }
}
