use axum::{Json, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use spl_token::instruction::mint_to;
use std::str::FromStr;
use base64;

use crate::routes::keypair::ApiResponse;

#[derive(Deserialize, Debug)]
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
    if body.mint.trim().is_empty()
        || body.destination.trim().is_empty()
        || body.authority.trim().is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("All fields (mint, destination, authority) are required")),
        ));
    }

    if body.amount == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Amount must be greater than 0")),
        ));
    }

    let mint = parse_pubkey(&body.mint, "mint")?;
    let dest = parse_pubkey(&body.destination, "destination")?;
    let authority = parse_pubkey(&body.authority, "authority")?;
    let instruction = mint_to(
        &spl_token::id(),
        &mint,
        &dest,
        &authority,
        &[],
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

fn parse_pubkey(key: &str, field_name: &str) -> Result<Pubkey, (StatusCode, Json<ApiResponse<()>>)> {
    Pubkey::from_str(key).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Invalid base58 public key in field: {}", field_name))),
        )
    })
}
