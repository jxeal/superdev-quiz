use axum::{Json, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use base64;
use bs58;
use std::convert::TryInto;

use ed25519_dalek::{Signature, VerifyingKey};
use ed25519_dalek::Verifier;

use crate::routes::keypair::ApiResponse;

#[derive(Deserialize)]
pub struct VerifyMessageRequest {
    pub message: String,
    pub signature: String, // base64
    pub pubkey: String,    // base58
}

#[derive(Serialize)]
pub struct VerifyMessageResponse {
    pub valid: bool,
    pub message: String,
    pub pubkey: String,
}

pub async fn verify_message(
    Json(body): Json<VerifyMessageRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    // Decode base64 signature
    let sig_bytes = base64::decode(&body.signature).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Invalid base64 signature")),
        )
    })?;

    let sig_array: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("Signature must be 64 bytes")),
            )
        })?;

    let signature = Signature::try_from(&sig_array[..]).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Malformed signature bytes")),
        )
    })?;

    // Decode base58 public key
    let pubkey_bytes = bs58::decode(&body.pubkey).into_vec().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Invalid base58 public key")),
        )
    })?;

    // Convert to [u8; 32]
    let pubkey_array: [u8; 32] = pubkey_bytes
        .try_into()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("Public key must be 32 bytes")),
            )
        })?;

    let verifying_key = VerifyingKey::from_bytes(&pubkey_array).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Malformed public key")),
        )
    })?;

    // Verify the signature
    let valid = VerifyingKey::verify(&verifying_key, body.message.as_bytes(), &signature).is_ok();

    let response = VerifyMessageResponse {
        valid,
        message: body.message,
        pubkey: body.pubkey,
    };

    Ok(Json(ApiResponse::success(response)))
}
