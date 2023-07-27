use std::str::FromStr;

use async_session::{
    chrono::{Duration, Utc},
    MemoryStore,
    Session,
    SessionStore,
};
use axum::{
    extract::State,
    headers,
    http::{
        header::{LOCATION, SET_COOKIE},
        HeaderMap,
        StatusCode,
    },
    response::IntoResponse,
    Json,
    TypedHeader,
};
use biscuit::{
    jwa::{
        ContentEncryptionAlgorithm,
        EncryptionOptions,
        KeyManagementAlgorithm,
        SignatureAlgorithm,
    },
    jwe::{self, Compact},
    jws::{self, Secret},
    ClaimsSet,
    Empty,
    RegisteredClaims,
    JWE,
    JWT,
};
use futures::TryFutureExt;
use pockety::{
    models::PocketItem,
    GetRequestTokenResponse as PocketyGetRequestTokenResponse,
    Pockety,
    PocketyUrl,
};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};

use crate::{
    error::{self, Error},
    ApiResult,
    AppState,
    Config,
    SessionData,
    TypedResponse,
    COOKIE_NAME,
};

pub async fn health_check() -> impl IntoResponse {
    "Healthy!"
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRequestTokenResponse {
    request_token: String,
    auth_uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OAuthStateParam {
    pub request_token: String,
    pub session_id: String,
    pub csrf_token: String,
}

pub async fn get_request_token(
    State(store): State<MemoryStore>,
    State(pockety): State<Pockety>,
    State(config): State<Config>,
) -> ApiResult<()> {
    // TODO: implement rate limiting
    // TODO: is there a way to check if a user is already authed?
    // TODO: use an actual cookie manager, since we're currently not signing or encrypting them
    let PocketyGetRequestTokenResponse { code, .. } = pockety.get_request_token(None).await?;

    let mut csrf_token = [0u8; 32];
    thread_rng().fill_bytes(&mut csrf_token);
    let csrf_token: String = csrf_token.iter().map(|t| t.to_string()).collect();

    let mut session = Session::new();
    let session_id = session.id().to_string();

    let session_data = SessionData {
        request_token: Some(code.clone()),
        csrf_token: Some(csrf_token.clone()),
        ..Default::default()
    };

    tracing::debug!("generated following session_data: {session_data:#?}");

    // session
    session.insert("session", &session_data)?;
    store.store_session(session).await?;

    // state
    let state = OAuthStateParam {
        request_token: code.clone(),
        session_id,
        csrf_token,
    };

    // sign the token
    let claims = ClaimsSet::<OAuthStateParam> {
        registered: RegisteredClaims {
            issuer: Some(FromStr::from_str("https://just-links.dev").unwrap()),
            not_before: Some(Utc::now().timestamp().into()),
            ..Default::default()
        },
        private: state,
    };

    let jwt = JWT::new_decoded(
        From::from(jws::RegisteredHeader {
            algorithm: SignatureAlgorithm::HS256,
            ..Default::default()
        }),
        claims,
    );

    let jws = jwt.into_encoded(&Secret::Bytes(config.jws_signing_secret.into()))?;

    let mut nonce = [0u8; 12];
    thread_rng().fill_bytes(&mut nonce);
    let options = EncryptionOptions::AES_GCM {
        nonce: nonce.to_vec(),
    };

    // Construct the JWE
    let jwe = JWE::new_decrypted(
        From::from(jwe::RegisteredHeader {
            cek_algorithm: KeyManagementAlgorithm::A256GCMKW,
            enc_algorithm: ContentEncryptionAlgorithm::A256GCM,
            media_type: Some("JOSE".to_string()),
            content_type: Some("JOSE".to_string()),
            ..Default::default()
        }),
        jws,
    );

    // Encrypt
    if let Ok(Compact::Encrypted(token)) = jwe.encrypt(&config.jwe_encryption_key, &options) {
        let auth_uri = format!(
            "{}?request_token={}&redirect_uri={}&state={}",
            PocketyUrl::AUTHORIZE,
            code,
            pockety.redirect_url,
            token.to_string()
        );

        let mut headers = HeaderMap::new();
        headers.insert(LOCATION, auth_uri.parse().unwrap());

        Ok(TypedResponse::new(None)
            .headers(Some(headers))
            .status_code(StatusCode::SEE_OTHER))
    } else {
        Err(Error::Internal("Failed to encrypt token".to_string()))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccessTokenRequest {
    state: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccessTokenResponse {
    username: String,
    state: Option<String>,
}

#[axum::debug_handler(state = AppState)]
pub async fn get_access_token(
    State(pockety): State<Pockety>,
    State(store): State<MemoryStore>,
    State(config): State<Config>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
    session_data: SessionData,
    body: Json<GetAccessTokenRequest>,
) -> ApiResult<GetAccessTokenResponse> {
    // get state from request body and decode it
    if let Some(token) = body.state.clone() {
        // ... some time later, we get token back!
        let token: JWE<OAuthStateParam, Empty, Empty> = JWE::new_encrypted(&token);

        // Decrypt
        let decrypted_jwe = token
            .into_decrypted(
                &config.jwe_encryption_key,
                KeyManagementAlgorithm::A256GCMKW,
                ContentEncryptionAlgorithm::A256GCM,
            )
            .unwrap();

        let decrypted_jws = decrypted_jwe.payload().unwrap();
        println!("decrypted_jws: {:#?}", decrypted_jws);
    } else {
        return Err(Error::BadRequest("Missing state param".to_string()));
    }

    if let Some(request_token) = session_data.request_token.clone() {
        // TODO: fill in the state param
        pockety
            .get_access_token(request_token.clone(), None)
            .and_then(|res| async move {
                // destroy old session
                // TODO: no need to destroy old session. just load and insert
                let cookie = cookies.get(COOKIE_NAME).unwrap();
                let session = store
                    .load_session(cookie.to_string())
                    .await
                    .unwrap()
                    .unwrap();
                store.destroy_session(session).await.unwrap();

                // create new session
                let mut session = Session::new();
                session
                    .insert(
                        "session",
                        &SessionData {
                            request_token: Some(request_token),
                            access_token: Some(res.access_token.clone()),
                            username: Some(res.username.clone()),
                            csrf_token: None
                        },
                    )
                    .unwrap();

                let cookie_expiration = Utc::now() + Duration::hours(1);
                let cookie = store
                    .store_session(session)
                    .await
                    .ok()
                    .flatten()
                    .ok_or(Error::Cookie("failed to store session".to_string()))
                    .unwrap();
                let cookie = format!("{COOKIE_NAME}={cookie}; Expires={cookie_expiration}; SameSite=Lax; HttpOnly; Secure");

                Ok((res, cookie))
            })
            .map_ok(|(res, cookie)| {
                let mut headers = HeaderMap::new();
                headers.insert(SET_COOKIE, cookie.parse().unwrap());

                TypedResponse::new(Some(GetAccessTokenResponse {
                    username: res.username,
                    state: res.state,
                }))
                .headers(Some(headers))
            })
            .map_err(Error::from)
            .await
    } else {
        Err(error::Error::Pocket(format!(
            "I couldn't find your request token"
        )))
    }
}

#[derive(Serialize)]
pub struct GetArticlesResponse {
    articles: Vec<PocketItem>,
}

pub async fn get_articles(
    State(pockety): State<Pockety>,
    session_data: SessionData,
) -> ApiResult<GetArticlesResponse> {
    if let Some(access_token) = session_data.access_token {
        let since = Utc::now() - Duration::days(7);

        pockety
            .retrieve()
            .access_token(access_token)
            .since(since)
            .execute()
            .map_ok(|articles| TypedResponse::new(Some(GetArticlesResponse { articles })))
            .map_err(Error::from)
            .await
    } else {
        Err(error::Error::Pocket(format!(
            "I couldn't find your access_token"
        )))
    }
}

#[derive(Serialize)]
pub struct GetSessionInfoResponse {
    pub username: Option<String>,
}

pub async fn get_session_info(session_data: SessionData) -> TypedResponse<GetSessionInfoResponse> {
    TypedResponse::new(Some(GetSessionInfoResponse {
        username: session_data.username,
    }))
}
