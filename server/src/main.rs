use std::{env, net::SocketAddr};

use async_session::MemoryStore;
use axum::{
    http::Method,
    routing::{get, post},
    Router,
    Server,
};
use dotenvy::dotenv;
use just_links_api::{
    api::{get_access_token, get_articles, get_request_token, health_check},
    AppState,
};
use pockety::Pockety;
use rand::{thread_rng, RngCore};
use tower_http::{cors::CorsLayer, trace};
use tracing::Level;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let store = MemoryStore::new();
    let mut secret = [0; 64];
    thread_rng()
        .try_fill_bytes(&mut secret)
        .expect("Failed to generate secret");
    let cors_layer = CorsLayer::new()
        .allow_origin([
            "http://localhost:5173".parse().unwrap(),
            "https://getpocket.com".parse().unwrap(),
        ])
        .allow_headers(["content-type".parse().unwrap()])
        .allow_methods([Method::GET, Method::POST])
        .allow_credentials(true);

    let pockety = Pockety::new(
        &env::var("POCKET_CONSUMER_KEY").expect("Missing POCKET_CONSUMER_KEY"),
        &env::var("POCKET_REDIRECT_URI").expect("Missing POCKET_REDIRECT_URI"),
    );

    let app_state = AppState { pockety, store };

    let app = Router::new()
        .route("/", get(health_check))
        .route("/articles", get(get_articles))
        .route("/auth/pocket", post(get_request_token))
        .route("/auth/authorize", post(get_access_token))
        .layer(cors_layer)
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Listening on {addr}");

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to launch server");
}
