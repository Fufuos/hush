use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptedSecret {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}
