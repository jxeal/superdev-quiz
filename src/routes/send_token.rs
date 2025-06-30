use axum::{Json, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use bs58;
use base64;
use solana_sdk::{
    instruction::AccountMeta,
    pubkey::Pubkey,
    instruction::Instruction,
};
use spl_token::instruction::transfer_checked;
use crate::routes::keypair::ApiResponse;

#[derive(Deserialize)]
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
    // Parse inputs to Pubkey
    let destination = parse_pubkey(&body.destination, "destination")?;
    let mint = parse_pubkey(&body.mint, "mint")?;
    let owner = parse_pubkey(&body.owner, "owner")?;

    // Derive token accounts (this is usually done with associated token accounts in real usage)
    // Here we mock token accounts assuming they're passed directly (not derived)

    // SPL Token program ID
    let token_program_id = spl_token::ID;

    // Normally you need associated token accounts; here we assume
    // destination is token account and mint matches what’s expected

    let decimals: u8 = 6; // default SPL token decimals (this should be passed/queried ideally)

    let ix = transfer_checked(
        &token_program_id,
        &owner,             // source token account
        &mint,              // mint
        &destination,       // destination token account
        &owner,             // authority
        &[],                // multisig signers
        body.amount,
        decimals,
    )
    .map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Failed to create token transfer instruction")),
        )
    })?;

    let accounts = ix
        .accounts
        .iter()
        .map(|acc| TokenAccountMeta {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let encoded_data = base64::encode(ix.data);

    let response = SendTokenResponse {
        program_id: ix.program_id.to_string(),
        accounts,
        instruction_data: encoded_data,
    };

    Ok(Json(ApiResponse::success(response)))
}

// Utility to decode and validate base58 pubkeys
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
