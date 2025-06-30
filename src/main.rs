
use axum::{routing::post, Router};

mod routes;
use routes::{
    keypair::keypair,
    create_token::create_token,
    token_mint::mint_token,
    message_sign::sign_message,
    message_verify::verify_message,
    send_sol::send_sol,
    send_token::send_token,
};


#[tokio::main]
async fn main() {
    print!("Hello");
    let app = Router::new()
        .route("/keypair", post(keypair))
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
        .route("/message/sign", post(sign_message))
        .route("/message/verify", post(verify_message))
        .route("/send/sol", post(send_sol))
        .route("/send/token", post(send_token));

    let addr = "0.0.0.0:7878";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener,app).await.unwrap();
}

