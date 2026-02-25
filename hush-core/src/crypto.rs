use crate::error::HushError;
use crate::models::EncryptedSecret;
use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
};

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

/// Lower-level decryption using raw slices for maximum flexibility
pub fn decrypt(ciphertext: &[u8], nonce_bytes: &[u8], key: &[u8; 32]) -> Result<String, HushError> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    if nonce_bytes.len() != 12 {
        return Err(HushError::DecryptionError);
    }
    let nonce = Nonce::from_slice(nonce_bytes);

    let decrypted_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| HushError::DecryptionError)?;

    String::from_utf8(decrypted_bytes).map_err(|_| HushError::DecryptionError)
}
