use std::{env, net::SocketAddr};

use app_server::{
    api::{
        articles::get_articles,
        auth::{get_access_token, get_request_token},
        health_check,
    },
    oauth::OAuthState,
    AppState,
    Config,
};
use async_session::MemoryStore;
use axum::{
    http::Method,
    routing::{get, post},
    Router,
    Server,
};
use biscuit::jwk::JWK;
use dotenvy::dotenv;
use pockety::Pockety;
use tower_http::{cors::CorsLayer, trace};
use tracing::Level;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter("app_server=debug")
        .compact()
        .init();

    let jws_signing_secret = biscuit::jws::Secret::Bytes(
        env::var("JWS_SIGNING_SECRET")
            .expect("Missing JWS_SIGNING_SECRET")
            .as_bytes()
            .to_vec(),
    );
    let jwe_encryption_key: JWK<OAuthState> = JWK::new_octet_key(
        env::var("JWE_ENCRYPTION_KEY")
            .expect("Missing JWE_ENCRYPTION_KEY")
            .as_bytes(),
        Default::default(),
    );

    let config = Config {
        jws_signing_secret,
        jwe_encryption_key,
    };

    let store = MemoryStore::new();

    let cors_layer = CorsLayer::new()
        .allow_origin([
            "http://localhost:5173".parse().unwrap(),
            "http://127.0.0.1:5173".parse().unwrap(),
            "https://getpocket.com".parse().unwrap(),
        ])
        .allow_headers([
            "Content-Type".parse().unwrap(),
            "Authorization".parse().unwrap(),
            "Accept".parse().unwrap(),
            "Origin".parse().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST])
        .allow_credentials(true);

    let pocket_consumer_key = env::var("POCKET_CONSUMER_KEY").expect("Missing POCKET_CONSUMER_KEY");
    let pocket_redirect_uri = env::var("POCKET_REDIRECT_URI").expect("Missing POCKET_REDIRECT_URI");
    tracing::debug!("Initializing Pockety instance with consumer_key: {pocket_consumer_key} and redirect_uri: {pocket_redirect_uri}.");

    let pockety = Pockety::new(pocket_consumer_key, pocket_redirect_uri.as_str())
        .expect("Failed to initialize Pockety instance.");

    let app_state = AppState {
        pockety,
        store,
        config,
    };

    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/articles", get(get_articles))
        .route("/auth/authn", post(get_request_token))
        .route("/auth/authz", post(get_access_token))
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
