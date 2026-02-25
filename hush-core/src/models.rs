use crate::crypto;
use crate::error::HushError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptedSecret {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl EncryptedSecret {
    /// Convenience method for when you already have the struct
    pub fn decrypt(&self, key: &[u8; 32]) -> Result<String, HushError> {
        crypto::decrypt(&self.ciphertext, &self.nonce, key)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretResponse {
    pub id: String,
    pub name: String,
    pub secret: EncryptedSecret,
}
