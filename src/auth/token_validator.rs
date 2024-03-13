// use azure_cosmosdb::prelude::*;
use base64::{engine::general_purpose, Engine};
use jsonwebtoken::{decode, DecodingKey, Validation};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    upn: String,
    tid: String,
    iss: String,
    // Add other relevant claims from the token
}
#[derive(Clone)]
pub struct TokenValidator {
    // cosmos_client: CosmosClient,
    signup_metadata_url: String,
    signin_metadata_url: String,
    decoding_keys: Arc<RwLock<HashMap<String, DecodingKey>>>,
}

impl TokenValidator {
    pub fn new(// cosmos_client: CosmosClient,
        // signup_metadata_url: String,
        // signin_metadata_url: String,
    ) -> Self {
        TokenValidator {
            // cosmos_client,
            signup_metadata_url: "someurl".to_string(),
            signin_metadata_url: "someurl".to_string(),
            decoding_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn validate_token(&self, token: &str) -> Result<User, String> {
        let claims = decode_token(token)?;
        let decoding_key = self.get_decoding_key(&claims.iss).await?;
        let validation = Validation::default();

        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                let upn = token_data.claims.upn;
                self.find_user_by_upn(&upn)
            }
            Err(_) => Err("Invalid token".to_string()),
        }
    }

    async fn get_decoding_key(&self, iss: &str) -> Result<DecodingKey, String> {
        let metadata_url = if iss.contains("signup") {
            &self.signup_metadata_url
        } else if iss.contains("signin") {
            &self.signin_metadata_url
        } else {
            return Err("Invalid issuer".to_string());
        };

        let kid = extract_kid(iss)?;
        let cache_key = format!("{}:{}", metadata_url, kid);

        {
            let decoding_keys = self.decoding_keys.read().unwrap();
            if let Some(decoding_key) = decoding_keys.get(&cache_key) {
                return Ok(decoding_key.clone());
            }
        }

        let decoding_key = get_decoding_key(metadata_url, &kid).await?;
        {
            let mut decoding_keys = self.decoding_keys.write().unwrap();
            decoding_keys.insert(cache_key, decoding_key.clone());
        }

        Ok(decoding_key)
    }

    fn find_user_by_upn(&self, upn: &str) -> Result<User, String> {
        if upn == "billy@yahoo.com" {
            Ok(User {
                id: "123".to_string(),
                upn: "billy@yahoo.com".to_string(),
            })
        } else {
            Err("User Not Found".to_string())
        }
    }
    // async fn find_user_by_upn(&self, upn: &str) -> Result<User, String> {
    //     let query = format!("SELECT * FROM c WHERE c.upn = '{}'", upn);
    //     let options = QueryDocumentOptions::default();
    //
    //     match self
    //         .cosmos_client
    //         .query_documents("YourDatabaseId", "YourCollectionId", &query, options)
    //         .await
    //     {
    //         Ok(documents) => {
    //             if let Some(document) = documents.into_iter().next() {
    //                 let user: User = serde_json::from_value(document).unwrap();
    //                 Ok(user)
    //             } else {
    //                 Err("User not found".to_string())
    //             }
    //         }
    //         Err(_) => Err("Failed to query user".to_string()),
    //     }
    // }
}

fn decode_token(token: &str) -> Result<Claims, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid token format".to_string());
    }

    let claims_str = general_purpose::STANDARD
        .decode(parts[1])
        .map_err(|_| "Failed to decode claims")?;
    serde_json::from_slice(&claims_str).map_err(|_| "Failed to deserialize claims".to_string())
}

fn extract_kid(iss: &str) -> Result<String, String> {
    let parts: Vec<&str> = iss.split('/').collect();
    if parts.len() < 2 {
        return Err("Invalid issuer format".to_string());
    }

    Ok(parts[parts.len() - 2].to_string())
}

async fn get_decoding_key(metadata_url: &str, kid: &str) -> Result<DecodingKey, String> {
    let response = reqwest::get(metadata_url)
        .await
        .map_err(|_| "Failed to fetch metadata")?;
    let metadata: serde_json::Value = response
        .json()
        .await
        .map_err(|_| "Failed to parse metadata")?;

    let keys = metadata["keys"]
        .as_array()
        .ok_or("Invalid metadata format")?;
    let key = keys
        .iter()
        .find(|key| key["kid"] == kid)
        .ok_or("Key not found")?;
    let x5c = key["x5c"][0].as_str().ok_or("Missing x5c")?;

    let certificate = x5c
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("");
    let certificate_der = general_purpose::STANDARD
        .decode(&certificate)
        .map_err(|_| "Failed to decode certificate")?;

    Ok(DecodingKey::from_ec_der(&certificate_der))
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    upn: String,
    // Add other relevant user fields
}
