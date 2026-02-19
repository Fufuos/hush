use thiserror::Error;

#[derive(Error, Debug)]
pub enum HushError {
    #[error("Encryption failed")]
    EncryptionError,

    #[error("Decryption failed - possibly wrong key or corrupted data")]
    DecryptionError,

    #[error("Invalid key length: expected 32 bytes")]
    InvalidKeyLength,

    #[error("Internal error: {0}")]
    Internal(String),
}
