use std::str::FromStr;

use async_session::{
    chrono::{Duration, Utc},
    MemoryStore,
    Session,
    SessionStore,
};
use axum::{
    extract::State,
    http::{
        header::{LOCATION, SET_COOKIE},
        HeaderMap,
        StatusCode,
    },
    response::IntoResponse,
    Json,
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
    GetAccessTokenResponse as PocketyGetAccessTokenResponse,
    GetRequestTokenResponse as PocketyGetRequestTokenResponse,
    Pockety,
    PocketyUrl,
};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};

use crate::{
    error::{self, ApiError, Error},
    ApiResult,
    AppState,
    Config,
    SessionData,
    TypedResponse,
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
    // TODO: use session_id instead of session cookie.
    // we're only using the cookie here because async_session only allows access
    // SessionStore using a cookie.
    pub session_cookie: String,
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
    let session_data = SessionData {
        request_token: Some(code.clone()),
        csrf_token: Some(csrf_token.clone()),
        ..Default::default()
    };

    tracing::debug!("generated following session_data: {session_data:#?}");

    // session
    // TODO: abstract this probably, since we don't want to handle CRUD with session
    // through raw string keys every time right?
    session.insert("session", &session_data)?;
    let session_cookie = store
        .store_session(session)
        .await
        .ok()
        .flatten()
        .ok_or(Error::Api(ApiError::InternalServerError(
            "Failed to store session".to_string(),
        )))?;

    // state
    let state = OAuthStateParam {
        request_token: code.clone(),
        session_cookie,
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
        Err(Error::Api(ApiError::InternalServerError(
            "Failed to encrypt token".to_string(),
        )))
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAccessTokenRequest {
    state: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
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
    body: Json<GetAccessTokenRequest>,
) -> ApiResult<GetAccessTokenResponse> {
    let state = if let Some(token) = body.state.clone() {
        let token: JWE<OAuthStateParam, Empty, Empty> = JWE::new_encrypted(&token);

        match token
            .into_decrypted(
                &config.jwe_encryption_key,
                KeyManagementAlgorithm::A256GCMKW,
                ContentEncryptionAlgorithm::A256GCM,
            )
            .and_then(|d| d.payload().cloned())?
        {
            jws::Compact::Decoded { payload, .. } => {
                println!("payload: {payload:#?}");
                payload
            }
            jws::Compact::Encoded(_) => {
                return Err(Error::Api(ApiError::InternalServerError(
                    "Failed to decrypt token".to_string(),
                )))?;
            }
        }
    } else {
        return Err(Error::Api(ApiError::BadRequest(
            "Missing state param".to_string(),
        )))?;
    };

    let OAuthStateParam {
        request_token,
        session_cookie,
        csrf_token,
    } = state.private.clone();

    // TODO: Should I fill `state` in?
    let res: PocketyGetAccessTokenResponse = pockety
        .get_access_token(request_token.clone(), None)
        .map_err(Error::from)
        .await?;

    let mut session = store
        .clone()
        .load_session(session_cookie.clone())
        .await
        .ok()
        .flatten()
        .ok_or(Error::Api(ApiError::InternalServerError(
            "Couldn't find the session".to_string(),
        )))?;

    // retrieve session by cookie value
    let mut session_data: SessionData =
        session
            .get("session")
            .ok_or(Error::Api(ApiError::InternalServerError(
                "Empty session".to_string(),
            )))?;

    // compare csrf_token in request with csrf_token in session
    if csrf_token
        != session_data
            .csrf_token
            .clone()
            .ok_or(Error::Api(ApiError::InternalServerError(
                "CSRF token missing in session".to_string(),
            )))?
    {
        return Err(Error::Api(ApiError::Unauthorized(
            "CSRF token doesn't match".to_string(),
        )));
    }

    session_data.access_token = Some(res.access_token.clone());
    session.insert("session", &session_data)?;

    // update session
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, session_cookie.parse().unwrap());

    Ok(TypedResponse::new(Some(GetAccessTokenResponse {
        username: res.username.clone(),
        state: res.state,
    }))
    .headers(Some(headers)))
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
