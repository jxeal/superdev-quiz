use axum::{Json, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use solana_sdk::signature::{Keypair, Signer};
use base64;
use bs58;

use crate::routes::keypair::ApiResponse;

#[derive(Deserialize)]
pub struct SignMessageRequest {
    pub message: String,
    pub secret: String, // base58-encoded 64-byte secret key
}

#[derive(Serialize)]
pub struct SignMessageResponse {
    pub signature: String,
    pub public_key: String,
    pub message: String,
}

pub async fn sign_message(
    Json(body): Json<SignMessageRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    // Decode the base58 secret key
    let secret_bytes = bs58::decode(&body.secret)
        .into_vec()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("Invalid base58-encoded secret key")),
            )
        })?;

    if secret_bytes.len() != 64 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Secret key must be 64 bytes")),
        ));
    }

    // Create Keypair from bytes
    let keypair = Keypair::from_bytes(&secret_bytes).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Failed to parse secret key into Keypair")),
        )
    })?;

    // Sign the message
    let signature = keypair.sign_message(body.message.as_bytes());
    let response = SignMessageResponse {
        signature: base64::encode(signature.as_ref()),
        public_key: keypair.pubkey().to_string(),
        message: body.message,
    };

    Ok(Json(ApiResponse::success(response)))
}
