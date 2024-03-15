use async_graphql::{Data, Result};
use axum::http::HeaderMap;
use serde::Deserialize;

pub struct Token(pub String);

#[derive(Deserialize)]
struct Payload {
    #[serde(rename = "Authorization")]
    authorization: String,
}

#[derive(Clone)]
pub struct Auth;

impl Auth {
    pub fn get_token_from_headers(headers: &HeaderMap) -> Option<Token> {
        headers
            .get("Authorization")
            .and_then(|value| value.to_str().map(|s| Token(s.to_string())).ok())
    }

    pub async fn on_connection_init(value: serde_json::Value) -> Result<Data> {
        if let Ok(payload) = serde_json::from_value::<Payload>(value) {
            let mut data = Data::default();
            data.insert(Token(payload.authorization));
            Ok(data)
        } else {
            Err("Token is required".into())
        }
    }
}
