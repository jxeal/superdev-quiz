use axum::{Json, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use spl_token::instruction::initialize_mint;
use std::str::FromStr;
use base64;

use crate::routes::keypair::ApiResponse;

#[derive(Deserialize, Debug)]
pub struct CreateTokenRequest {
    #[serde(rename = "mintAuthority")]
    pub mint_authority: String,

    #[serde(rename = "mint")]
    pub mint: String,

    pub decimals: u8,
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

pub async fn create_token(
    Json(body): Json<CreateTokenRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    if body.mint.trim().is_empty() || body.mint_authority.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Mint and Mint Authority fields cannot be empty")),
        ));
    }

    if body.decimals > 18 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Decimals must be between 0 and 18")),
        ));
    }

    let mint_pubkey = parse_pubkey(&body.mint)?;
    let mint_authority = parse_pubkey(&body.mint_authority)?;

    let instruction = initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &mint_authority,
        None,
        body.decimals,
    )
    .map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Failed to create instruction: {}", e))),
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
