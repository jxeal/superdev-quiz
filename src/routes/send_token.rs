use axum::{Json, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use bs58;
use base64;
use solana_sdk::{
    pubkey::Pubkey,
    instruction::Instruction,
};
use spl_token::instruction::transfer_checked;

use crate::routes::keypair::ApiResponse;

#[derive(Deserialize, Debug)]
pub struct SendTokenRequest {
    pub destination: String,
    pub mint: String,
    pub owner: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct TokenAccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Serialize)]
pub struct SendTokenResponse {
    pub program_id: String,
    pub accounts: Vec<TokenAccountMeta>,
    pub instruction_data: String,
}

pub async fn send_token(
    Json(body): Json<SendTokenRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    if body.amount == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Amount must be greater than 0")),
        ));
    }

    let destination = parse_pubkey(&body.destination, "destination")?;
    let mint = parse_pubkey(&body.mint, "mint")?;
    let owner = parse_pubkey(&body.owner, "owner")?;

    let token_program_id = spl_token::ID;
    let decimals: u8 = 6; 
    let ix = transfer_checked(
        &token_program_id,
        &owner,       
        &mint,
        &destination,  
        &owner,        
        &[],          
        body.amount,
        decimals,
    )
    .map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Failed to create token transfer instruction")),
        )
    })?;

    let accounts = ix.accounts.into_iter().map(|acc| TokenAccountMeta {
        pubkey: acc.pubkey.to_string(),
        is_signer: acc.is_signer,
        is_writable: acc.is_writable,
    }).collect();

    let encoded_data = base64::encode(ix.data);

    let response = SendTokenResponse {
        program_id: ix.program_id.to_string(),
        accounts,
        instruction_data: encoded_data,
    };

    Ok(Json(ApiResponse::success(response)))
}

fn parse_pubkey(input: &str, field: &str) -> Result<Pubkey, (StatusCode, Json<ApiResponse<()>>)> {
    let bytes = bs58::decode(input).into_vec().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Invalid base58 in {}", field))),
        )
    })?;

    Pubkey::try_from(bytes.as_slice()).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("{} must be 32 bytes", field))),
        )
    })
}
