use axum::{Json, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use bs58;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
    system_instruction,
};

use crate::routes::keypair::ApiResponse;

#[derive(Deserialize)]
pub struct SendSolRequest {
    pub from: String,
    pub to: String,
    pub lamports: u64,
}

#[derive(Serialize)]
pub struct SendSolResponse {
    pub program_id: String,
    pub accounts: Vec<String>,
    pub instruction_data: String,
}

pub async fn send_sol(
    Json(body): Json<SendSolRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    // Parse `from` public key
    let from_pubkey = bs58::decode(&body.from)
        .into_vec()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("Invalid sender public key")),
            )
        })
        .and_then(|bytes| {
            Pubkey::try_from(bytes.as_slice()).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error("Sender public key must be 32 bytes")),
                )
            })
        })?;

    // Parse `to` public key
    let to_pubkey = bs58::decode(&body.to)
        .into_vec()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("Invalid recipient public key")),
            )
        })
        .and_then(|bytes| {
            Pubkey::try_from(bytes.as_slice()).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error("Recipient public key must be 32 bytes")),
                )
            })
        })?;

    // Create transfer instruction
    let instruction: Instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, body.lamports);

    let account_addresses: Vec<String> = instruction
        .accounts
        .iter()
        .map(|acc| acc.pubkey.to_string())
        .collect();

    let instruction_data = base64::encode(&instruction.data);

    let response = SendSolResponse {
        program_id: instruction.program_id.to_string(),
        accounts: account_addresses,
        instruction_data,
    };

    Ok(Json(ApiResponse::success(response)))
}
