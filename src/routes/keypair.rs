use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use solana_sdk::signature::{Keypair, Signer};
use bs58;

#[derive(Serialize)]
struct KeypairResponse {
    pub pubkey: String,
    pub secret: String,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.to_string()),
        }
    }
}

pub async fn keypair() -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey().to_string(); // already base58
    let secret = bs58::encode(keypair.to_bytes()).into_string();

    let response = KeypairResponse { pubkey, secret };

    Ok(Json(ApiResponse::success(response)))
}
