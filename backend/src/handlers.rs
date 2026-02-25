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
#[derive(Deserialize)]
pub struct UpdateSecretRequest {
    pub new_name: String,
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

pub async fn list_secrets(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<Vec<SecretResponse>>, StatusCode> {
    // Notice we use fetch_all instead of fetch_optional
    let records = sqlx::query!("SELECT id, name, ciphertext, nonce FROM secrets")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Map the database rows into our JSON response structs
    let response: Vec<SecretResponse> = records
        .into_iter()
        .map(|row| SecretResponse {
            id: row.id,
            name: row.name,
            secret: EncryptedSecret {
                ciphertext: row.ciphertext,
                nonce: row.nonce,
            },
        })
        .collect();

    Ok(Json(response))
}

// Make sure you have `Path` imported at the top if it isn't already:
// use axum::extract::Path;

/// DELETE /secrets/{name}
pub async fn delete_secret(
    Path(name): Path<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query!("DELETE FROM secrets WHERE name = ?", name)
        .execute(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ Database error in delete_secret: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // If no rows were deleted, the secret didn't exist
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // 204 No Content is the standard HTTP response for a successful delete
    Ok(StatusCode::NO_CONTENT)
}

/// PUT /secrets/:name
pub async fn update_secret(
    Path(name): Path<String>,
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<UpdateSecretRequest>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query!(
        "UPDATE secrets SET name = ?, ciphertext = ?, nonce = ? WHERE name = ?",
        payload.new_name,
        payload.secret.ciphertext,
        payload.secret.nonce,
        name
    )
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}
