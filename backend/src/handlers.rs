use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use hush_core::EncryptedSecret;

#[derive(Deserialize)]
pub struct CreateSecretRequest {
    pub name: String,
    pub secret: EncryptedSecret,
}

#[derive(Serialize)]
pub struct SecretResponse {
    pub id: String,
    pub name: String,
    pub secret: EncryptedSecret,
}

/// POST /secrets
pub async fn create_secret(
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<CreateSecretRequest>,
) -> Result<(StatusCode, Json<SecretResponse>), StatusCode> {
    let id = Uuid::new_v4().to_string();

    // SQLx checks this query against your actual database schema at compile time!
    let result = sqlx::query!(
        "INSERT INTO secrets (id, name, ciphertext, nonce) VALUES (?, ?, ?, ?)",
        id,
        payload.name,
        payload.secret.ciphertext,
        payload.secret.nonce
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            let response = SecretResponse {
                id,
                name: payload.name,
                secret: payload.secret,
            };
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR), // E.g., if the name already exists
    }
}

/// GET /secrets/:name
pub async fn get_secret(
    Path(name): Path<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<SecretResponse>, StatusCode> {
    let record = sqlx::query!(
        "SELECT id, name, ciphertext, nonce FROM secrets WHERE name = ?",
        name
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match record {
        Some(row) => {
            let response = SecretResponse {
                id: row.id,
                name: row.name,
                secret: EncryptedSecret {
                    ciphertext: row.ciphertext,
                    nonce: row.nonce,
                },
            };
            Ok(Json(response))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}
