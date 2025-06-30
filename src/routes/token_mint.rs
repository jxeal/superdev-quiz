use axum::{Json, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use spl_token::instruction::mint_to;
use std::str::FromStr;
use base64;

use crate::routes::keypair::ApiResponse;

#[derive(Deserialize)]
pub struct MintTokenRequest {
    pub mint: String,
    pub destination: String,
    pub authority: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct AccountInfo {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Serialize)]
pub struct TokenInstructionResponse {
    pub program_id: String,
    pub accounts: Vec<AccountInfo>,
    pub instruction_data: String,
}

pub async fn mint_token(
    Json(body): Json<MintTokenRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    let mint = parse_pubkey(&body.mint)?;
    let dest = parse_pubkey(&body.destination)?;
    let authority = parse_pubkey(&body.authority)?;

    let instruction = mint_to(
        &spl_token::id(),
        &mint,
        &dest,
        &authority,
        &[], // multisig signer pubkeys (empty for single authority)
        body.amount,
    )
    .map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Failed to build mint_to instruction: {}", e))),
        )
    })?;

    let response = TokenInstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts: instruction
            .accounts
            .into_iter()
            .map(|meta| AccountInfo {
                pubkey: meta.pubkey.to_string(),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        instruction_data: base64::encode(&instruction.data),
    };

    Ok(Json(ApiResponse::success(response)))
}

fn parse_pubkey(key: &str) -> Result<Pubkey, (StatusCode, Json<ApiResponse<()>>)> {
    Pubkey::from_str(key).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Invalid base58 public key")),
        )
    })
}
