use axum::response::IntoResponse;

pub mod articles;
pub mod auth;

pub async fn health_check() -> impl IntoResponse {
    "Healthy!"
}
