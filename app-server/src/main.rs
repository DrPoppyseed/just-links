use std::{env, net::SocketAddr, sync::Arc};

use app_server::{
    api::{
        articles::get_articles,
        auth::{get_access_token, get_request_token, get_session},
        health_check,
    },
    oauth::OAuthState,
    AppState, Config,
};
use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, ORIGIN},
        Method,
    },
    routing::{get, post},
    Router, Server,
};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use biscuit::{jwk::JWK, jws::Secret};
use dotenvy::dotenv;
use pockety::Pockety;
use sqlx::{migrate, postgres::PgPoolOptions};
use tower_http::{cors::CorsLayer, trace};
use tracing::{debug, info, Level};

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .pretty()
        .with_ansi(false)
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter("app_server=debug")
        .compact()
        .init();

    let jws_signing_secret = Secret::Bytes(
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

    let redis_url: String = env::var("REDIS_URL").expect("Missing REDIS_URL");
    let redis_connection_manager =
        RedisConnectionManager::new(redis_url).expect("Failed to build redis connection manager");
    let redis_connection_pool = Pool::builder()
        .build(redis_connection_manager)
        .await
        .expect("Failed to build redis pool");
    debug!("Initialized Redis connection pool");

    let postgres_url: String = env::var("DATABASE_URL").expect("Missing DATABASE_URL");
    let postgres_connection_pool = PgPoolOptions::new()
        .connect_lazy(&postgres_url)
        .expect("Failed to build postgres connection pool");
    debug!("Initialized Postgres connection pool");

    migrate!("./migrations")
        .run(&postgres_connection_pool)
        .await
        .expect("Failed to migrate database");
    debug!("Migrated Postgres database");

    let user_agent_url = env::var("USER_AGENT_URL").expect("Missing USER_AGENT_URL");
    let cors_layer = CorsLayer::new()
        .allow_origin([
            user_agent_url.parse().unwrap(),
            "https://getpocket.com".parse().unwrap(),
        ])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION, ACCEPT, ORIGIN])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_credentials(true);

    let pocket_consumer_key = env::var("POCKET_CONSUMER_KEY").expect("Missing POCKET_CONSUMER_KEY");
    let pocket_redirect_uri = env::var("POCKET_REDIRECT_URI").expect("Missing POCKET_REDIRECT_URI");
    debug!("Initializing Pockety instance.");

    let pockety = Pockety::new(pocket_consumer_key, pocket_redirect_uri.as_str())
        .expect("Failed to initialize Pockety instance.");

    let app_state = AppState {
        pockety,
        session_store: Arc::new(redis_connection_pool),
        db: Arc::new(postgres_connection_pool),
        config,
    };

    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/articles", get(get_articles))
        .route("/auth/authn", post(get_request_token))
        .route("/auth/authz", post(get_access_token))
        .route("/auth/session", get(get_session))
        .layer(cors_layer)
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Listening on {addr}");

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to launch server");
}
