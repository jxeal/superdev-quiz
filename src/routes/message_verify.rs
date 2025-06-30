use axum::{Json, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use base64;
use bs58;
use std::convert::TryInto;

use ed25519_dalek::{Signature, VerifyingKey};
use ed25519_dalek::Verifier;

use crate::routes::keypair::ApiResponse;

#[derive(Deserialize, Debug)]
pub struct VerifyMessageRequest {
    pub message: String,
    pub signature: String, 
    pub pubkey: String,    
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
    if body.message.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Message must not be empty")),
        ));
    }

    let sig_bytes = base64::decode(&body.signature).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Invalid base64 signature")),
        )
    })?;

    if sig_bytes.len() != 64 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Signature must be 64 bytes")),
        ));
    }

    let sig_array: [u8; 64] = sig_bytes.try_into().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Failed to convert signature to fixed-size array")),
        )
    })?;

    let signature = Signature::try_from(&sig_array[..]).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Malformed signature bytes")),
        )
    })?;

    let pubkey_bytes = bs58::decode(&body.pubkey).into_vec().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Invalid base58 public key")),
        )
    })?;

    if pubkey_bytes.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Public key must be 32 bytes")),
        ));
    }

    let pubkey_array: [u8; 32] = pubkey_bytes.try_into().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Failed to convert public key to fixed-size array")),
        )
    })?;

    let verifying_key = VerifyingKey::from_bytes(&pubkey_array).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Malformed public key")),
        )
    })?;

    let valid = verifying_key.verify(body.message.as_bytes(), &signature).is_ok();

    let response = VerifyMessageResponse {
        valid,
        message: body.message,
        pubkey: body.pubkey,
    };

    Ok(Json(ApiResponse::success(response)))
}
