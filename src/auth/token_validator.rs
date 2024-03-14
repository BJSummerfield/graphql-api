use crate::models::User;
use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet};
use jsonwebtoken::{decode, decode_header, DecodingKey, Header, Validation};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
    signup_metadata_url: String,
    signin_metadata_url: String,
    // decoding_keys: Arc<RwLock<HashMap<String, DecodingKey>>>,
}

impl TokenValidator {
    pub fn new() -> Self {
        TokenValidator {
            signup_metadata_url:
                "https://samuraimdrdevelop.b2clogin.com/samuraimdrdevelop.onmicrosoft.com/discovery/v2.0/keys?p=B2C_1A_SMART_HRD_SIGNUP"
                    .to_string(),
            signin_metadata_url:
                "https://samuraimdrdevelop.b2clogin.com/samuraimdrdevelop.onmicrosoft.com/discovery/v2.0/keys?p=B2C_1A_SMART_HRD"
                    .to_string(),
            // decoding_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn validate_token(&self, token: &str) -> Result<User, String> {
        let jwks_reply: &str = r#"
        {"keys":[{"kid":"TygE3fI3bbOJ25c7K6QJYDvNeonXftOCWvIEdAVBrRE","use":"sig","kty":"RSA","e":"AQAB","n":"l6yXvDrwGWWVM22EswNGOKO8lOOpjIg1nYS2ATs4gD6-Nv8ojastEGAo1zclU3LXffHm4qSb9UeumNrnScZAWcjBu5yzUUBLppv-DLsiorKkye9gPuecOhBt2K_V-5_HSXcKTbNQiWu5Cv6Ffi3vJd_v5kzq38l_xmq9hrTX8OxPbS9GxW5VdbZSuj124XVeEGkjG5twKzh0ej3gmTTLolueo7vTYdhZuHWg6ymnJe_DMDYgZUoRQnlUgZBCWZ33TXS5b0qbdc0qxq20wgDhVZbIHvYLiW19XcmEGdD7NAhiH_ml_VtyC4UsSuP_6EwZGEofMFvFOoqaTpiqBZj9bQ"}]}
        "#;
        let jwks: JwkSet = serde_json::from_str(jwks_reply).unwrap();
        let header = decode_header(token).unwrap();
        let Some(kid) = header.kid else {
            return Err("Missing kid in token header".to_string());
        };
        let Some(jwk) = jwks.find(&kid) else {
            return Err("No matching JWK found for the given kid".into());
        };

        let decoding_key = match &jwk.algorithm {
            AlgorithmParameters::RSA(rsa) => {
                DecodingKey::from_rsa_components(&rsa.n, &rsa.e).map_err(|e| e.to_string())?
            }
            _ => unreachable!("algorithm should be a RSA in this example"),
        };

        let validation = {
            let mut validation = Validation::new(header.alg);
            validation.set_audience(&["6ed1f7e5-9797-4d7f-9e0a-eb6bc75052cf"]);
            validation.validate_exp = false;
            validation
        };

        let decoded_token =
            decode::<HashMap<String, serde_json::Value>>(token, &decoding_key, &validation)
                .map_err(|e| e.to_string());

        println!("{:#?}", decoded_token);

        Ok(User {
            id: "123".to_string(),
            upn: "billy@yahoo.com".to_string(),
        })
    }
    // pub async fn validate_token(&self, token: &str) -> Result<User, String> {
    //     let header =
    //         decode_header(token).map_err(|_| "Failed to decode token header".to_string())?;
    //
    //     let kid = header
    //         .kid
    //         .clone()
    //         .ok_or("Missing kid in token header".to_string())?;
    //
    //     // Try to validate the token using the signin user flow
    //     let signin_result = self
    //         .validate_token_with_user_flow(token, &header, &kid, "signin")
    //         .await;
    //
    //     if signin_result.is_ok() {
    //         return signin_result;
    //     }
    //
    //     // If signin validation fails, try to validate the token using the signup user flow
    //     let signup_result = self
    //         .validate_token_with_user_flow(token, &header, &kid, "signup")
    //         .await;
    //
    //     if signup_result.is_ok() {
    //         return signup_result;
    //     }
    //
    //     // If both signin and signup validation fail, return an error
    //     Err("Invalid token".to_string())
    // }

    async fn validate_token_with_user_flow(
        &self,
        token: &str,
        header: &Header,
        kid: &str,
        user_flow: &str,
    ) -> Result<User, String> {
        let metadata_url = if user_flow == "signup" {
            &self.signup_metadata_url
        } else {
            &self.signin_metadata_url
        };

        let jwks = self.fetch_jwks(metadata_url).await?;

        let jwk = jwks
            .find(kid)
            .ok_or("No matching JWK found for the given kid".to_string())?;

        let decoding_key = match &jwk.algorithm {
            AlgorithmParameters::RSA(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                .map_err(|_| "Failed to create decoding key".to_string())?,
            _ => return Err("Unsupported algorithm".to_string()),
        };

        let mut validation = Validation::new(header.alg);
        validation.set_audience(&[
            "https://samuraimdrdevelop.b2clogin.com/2911a429-834b-42e3-a164-020644ffacab/v2.0/"
                .to_string(),
        ]);
        validation.validate_exp = false;

        let decoded_token = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|_| "Failed to decode token".to_string())?;

        let email = &decoded_token.claims.email;
        self.find_user_by_email(email)
    }

    async fn fetch_jwks(&self, metadata_url: &str) -> Result<JwkSet, String> {
        let response = reqwest::get(metadata_url)
            .await
            .map_err(|_| "Failed to fetch metadata".to_string())?;

        let jwks_json = response
            .text()
            .await
            .map_err(|_| "Failed to read metadata response".to_string())?;

        serde_json::from_str(&jwks_json).map_err(|_| "Failed to parse JWKS".to_string())
    }

    fn find_user_by_email(&self, email: &str) -> Result<User, String> {
        println!("Finding user by email: {}", email);
        if email == "bjsummerfield+inviteduser1@live.com" {
            Ok(User {
                id: "123".to_string(),
                upn: email.to_string(),
            })
        } else {
            Err("User Not Found".to_string())
        }
    }
}
