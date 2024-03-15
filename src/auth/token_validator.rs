use crate::models::User;
use jsonwebtoken::jwk::{AlgorithmParameters, Jwk, JwkSet};
use jsonwebtoken::{decode, decode_header, DecodingKey, Header, Validation};
use reqwest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    email: String,
    iss: String,
    aud: String,
    exp: i64,
    nbf: i64,
}

#[derive(Clone)]
pub struct TokenValidator {
    metadata_urls_jwks: Arc<Mutex<Vec<(String, JwkSet)>>>,
}

impl TokenValidator {
    pub fn new() -> Self {
        TokenValidator {
            metadata_urls_jwks: Arc::new(Mutex::new(vec![
                (
                    "https://samuraimdrdevelop.b2clogin.com/samuraimdrdevelop.onmicrosoft.com/discovery/v2.0/keys?p=B2C_1A_SMART_HRD_SIGNUP".to_string(),
                    JwkSet { keys: vec![] },
                ),
                (
                    "https://samuraimdrdevelop.b2clogin.com/samuraimdrdevelop.onmicrosoft.com/discovery/v2.0/keys?p=B2C_1A_SMART_HRD".to_string(),
                    JwkSet { keys: vec![] },
                ),
            ])),
        }
    }

    pub async fn validate_token(&self, token: &str) -> Result<User, String> {
        let header =
            decode_header(token).map_err(|_| "Failed to decode token header".to_string())?;
        let kid = header
            .kid
            .clone()
            .ok_or("Missing kid in token header".to_string())?;

        let mut metadata_urls_jwks = self.metadata_urls_jwks.lock().await;

        for (metadata_url, jwks) in metadata_urls_jwks.iter_mut() {
            if let Some(jwk) = jwks.find(&kid) {
                println!("Found jwk in cache for kid: {}", kid);
                return self.validate_token_with_jwk(token, &header, jwk);
            } else {
                println!("JWK not found in cache for kid: {}", kid);
                let refreshed_jwks = Self::fetch_jwks(metadata_url).await?;
                *jwks = refreshed_jwks;

                if let Some(jwk) = jwks.find(&kid) {
                    println!("Found jwk in refreshed cache for kid: {}", kid);
                    return self.validate_token_with_jwk(token, &header, jwk);
                }
            }
        }

        Err("Invalid token".to_string())
    }

    fn validate_token_with_jwk(
        &self,
        token: &str,
        header: &Header,
        jwk: &Jwk,
    ) -> Result<User, String> {
        let decoding_key = match &jwk.algorithm {
            AlgorithmParameters::RSA(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                .map_err(|_| "Failed to create decoding key".to_string())?,
            _ => return Err("Unsupported algorithm".to_string()),
        };

        let mut validation = Validation::new(header.alg);
        validation.set_audience(&["6ed1f7e5-9797-4d7f-9e0a-eb6bc75052cf"]);
        validation.validate_exp = false;

        let decoded_token = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|_| "Failed to decode token".to_string())?;

        let email = decoded_token.claims.email;
        Ok(User {
            id: "123".to_string(),
            upn: email,
        })
    }

    async fn fetch_jwks(metadata_url: &str) -> Result<JwkSet, String> {
        println!("Fetching JWKS from {}", metadata_url);
        let response = reqwest::get(metadata_url)
            .await
            .map_err(|_| "Failed to fetch metadata".to_string())?;

        let jwks_json = response
            .text()
            .await
            .map_err(|_| "Failed to read metadata response".to_string())?;

        serde_json::from_str(&jwks_json).map_err(|_| "Failed to parse JWKS".to_string())
    }
}
