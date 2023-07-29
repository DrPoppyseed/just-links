use std::fmt;

use async_session::base64;
use biscuit::{
    jwa::{
        ContentEncryptionAlgorithm,
        EncryptionOptions,
        KeyManagementAlgorithm,
        SignatureAlgorithm,
    },
    jwe,
    jwk::JWK,
    jws::{self, Secret},
    ClaimsSet,
    CompactPart,
    Empty,
    RegisteredClaims,
    JWE,
    JWT,
};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};

use crate::error::Error;

const JUST_LINKS_ISSUER: &str = "https://just-links.dev";

pub struct JwsEncoded<T>(pub jws::Compact<ClaimsSet<T>, Empty>);

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct OAuthState {
    pub request_token: String,
    // TODO: use session_id instead of session cookie.
    // we're only using the cookie here because async_session only allows access
    // SessionStore using a cookie.
    pub session_cookie: String,
    pub csrf_token: String,
}

impl OAuthState {
    pub fn new(request_token: String, session_cookie: String, csrf_token: String) -> Self {
        Self {
            request_token,
            session_cookie,
            csrf_token,
        }
    }

    pub fn into_token(
        self,
        jws_secret: impl Into<Secret>,
        jwe_key: impl Into<JWK<Self>>,
    ) -> Result<String, Error> {
        Jwt::jws_encode(self, jws_secret).and_then(|signed| Jwt::jwe_encrypt(signed, jwe_key))
    }

    pub fn from_token(
        token: String,
        jws_secret: impl Into<Secret>,
        jwe_key: impl Into<JWK<Self>>,
    ) -> Result<Self, Error> {
        Jwt::jwe_decrypt(&token, jwe_key, jws_secret)
    }
}

pub fn generate_csrf_token() -> String {
    let mut bytes = [0u8; 256 / 8];
    thread_rng().fill_bytes(&mut bytes);
    base64::encode_config(bytes, base64::URL_SAFE_NO_PAD)
}

pub struct Jwt;

impl Jwt {
    pub fn jws_encode<T>(payload: T, jws_secret: impl Into<Secret>) -> Result<JwsEncoded<T>, Error>
    where
        ClaimsSet<T>: CompactPart,
    {
        let claims = ClaimsSet::<T> {
            registered: RegisteredClaims {
                issuer: Some(JUST_LINKS_ISSUER.to_string()),
                ..Default::default()
            },
            private: payload,
        };

        JWT::new_decoded(
            jws::RegisteredHeader {
                algorithm: SignatureAlgorithm::HS256,
                ..Default::default()
            }
            .into(),
            claims,
        )
        .into_encoded(&jws_secret.into())
        .map(|encoded| JwsEncoded(encoded))
        .map_err(|e| Error::Jwt(e.to_string()))
    }

    // TODO: we don't need ownership of jwe_key. figure out a way to only borrow it.
    pub fn jwe_encrypt<T>(jws: JwsEncoded<T>, jwe_key: impl Into<JWK<T>>) -> Result<String, Error>
    where
        T: Serialize,
        for<'de> T: Deserialize<'de>,
        ClaimsSet<T>: CompactPart,
    {
        let nonce = Self::generated_nonce();
        let options = EncryptionOptions::AES_GCM { nonce };

        JWE::new_decrypted(
            jwe::RegisteredHeader {
                cek_algorithm: KeyManagementAlgorithm::A256GCMKW,
                enc_algorithm: ContentEncryptionAlgorithm::A256GCM,
                ..Default::default()
            }
            .into(),
            jws.0,
        )
        .encrypt(&jwe_key.into(), &options)
        .map_err(|e| Error::Jwt(e.to_string()))
        .and_then(|jwe| match jwe {
            jwe::Compact::Encrypted(jwe) => Ok(jwe.encode()),
            jwe::Compact::Decrypted { .. } => {
                Err(Error::Jwt("Failed to encrypt token".to_string()))
            }
        })
    }

    // TODO: we don't need ownership of jws_secret. figure out a way to only borrow it.
    pub fn jwe_decrypt<T>(
        token: &str,
        jwe_key: impl Into<JWK<T>>,
        // TODO: since non-jwt payloads are considered valid, either make the jws_secret an option,
        // or make the non-jwt output error
        jws_secret: impl Into<Secret>,
    ) -> Result<T, Error>
    where
        T: Serialize + Clone + fmt::Debug,
        for<'de> T: Deserialize<'de>,
    {
        println!("token: {:?}", token);

        let jwe_decrypted = JWE::<T, Empty, Empty>::new_encrypted(token)
            .into_decrypted(
                &jwe_key.into(),
                KeyManagementAlgorithm::A256GCMKW,
                ContentEncryptionAlgorithm::A256GCM,
            )
            .and_then(|d| d.payload().cloned())?;

        match jwe_decrypted {
            jws::Compact::Decoded { payload, .. } => Ok(payload.private),
            jws::Compact::Encoded(_) => jwe_decrypted
                .into_decoded(&jws_secret.into(), SignatureAlgorithm::HS256)
                .and_then(|decoded| decoded.payload().cloned())
                .map_err(|e| Error::Jwt(e.to_string()))
                .map(|claims| claims.private),
        }
    }

    fn generated_nonce() -> Vec<u8> {
        let mut nonce = vec![0u8; 96 / 8];
        thread_rng().fill_bytes(&mut nonce);
        nonce
    }
}

#[cfg(test)]
mod test {
    use biscuit::{jwk::JWK, jws};
    use serde::{Deserialize, Serialize};

    use crate::oauth::Jwt;

    #[test]
    fn can_round_trip() {
        const SECRET: &str = "secret";
        let jws_secret = jws::Secret::Bytes(SECRET.into());

        // 32 byte randomly generated key
        // use something like `openssl rand -hex 16` to generate
        const KEY: &str = "i0N1ZdPuPFMjD/iJljE1p+JWZt/1uwSb";
        let jwe_key = JWK::new_octet_key(KEY.as_bytes(), Default::default());

        #[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
        struct Payload {
            name: String,
            msg: String,
        }

        let payload = Payload {
            name: "john doe".to_string(),
            msg: "where there is a will, there is a way".to_string(),
        };

        let signed = Jwt::jws_encode(payload.clone(), jws_secret.clone()).unwrap();
        let encrypted = Jwt::jwe_encrypt(signed, jwe_key.clone()).unwrap();

        let decrypted = Jwt::jwe_decrypt(&encrypted, jwe_key, jws_secret).unwrap();

        assert_eq!(payload, decrypted);
    }
}
