use crate::graphql::SchemaType;
use crate::models::User;
use async_graphql::{Data, Error, ErrorExtensions, Result, ServerError};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, http::HeaderMap};
use serde::Deserialize;

#[derive(Deserialize)]
struct Payload {
    #[serde(rename = "Authorization")]
    authorization: String,
}

pub struct Auth;

impl Auth {
    pub async fn http(
        State(schema): State<SchemaType>,
        req: GraphQLRequest,
        headers: HeaderMap,
    ) -> GraphQLResponse {
        match Self::extract_token(&headers) {
            Some(token) => Self::process_request_with_token(schema, req, token).await,
            None => Self::unauthorized_response(),
        }
    }

    fn extract_token(headers: &HeaderMap) -> Option<String> {
        headers
            .get("Authorization")
            .map(|value| value.to_str().unwrap().to_string())
    }

    async fn process_request_with_token(
        schema: SchemaType,
        req: GraphQLRequest,
        token: String,
    ) -> GraphQLResponse {
        match Self::authenticate(token) {
            Ok(user) => {
                let req = req.into_inner().data(user);
                schema.execute(req).await.into()
            }
            Err(error) => GraphQLResponse::from(async_graphql::Response::from_errors(vec![error])),
        }
    }

    fn unauthorized_response() -> GraphQLResponse {
        let response =
            async_graphql::Response::from_errors(vec![Self::create_unauthorized_error()]);
        GraphQLResponse::from(response)
    }

    fn authenticate(auth_value: String) -> Result<User, ServerError> {
        let token = auth_value.into();
        if Self::is_token_valid(&token) {
            let user = Self::create_user_from_token(&token);
            Ok(user)
        } else {
            Err(Self::create_unauthorized_error())
        }
    }

    fn is_token_valid(token: &String) -> bool {
        token == "validToken"
    }

    pub async fn on_connection_init(value: serde_json::Value) -> Result<Data> {
        if let Ok(payload) = serde_json::from_value::<Payload>(value) {
            match Self::authenticate(payload.authorization) {
                Ok(user) => {
                    let mut data = Data::default();
                    data.insert(user);
                    Ok(data)
                }
                Err(_) => Err("Unauthorized".into()),
            }
        } else {
            Err("Token is required".into())
        }
    }

    fn create_unauthorized_error() -> ServerError {
        let error = Error::new("Unauthorized")
            .extend_with(|_, e| e.set("status", 401))
            .extend_with(|_, e| e.set("code", "UNAUTHORIZED"));
        ServerError {
            message: error.message,
            source: error.source,
            locations: Vec::new(),
            path: Vec::new(),
            extensions: error.extensions,
        }
    }

    fn create_user_from_token(_token: &String) -> User {
        User::new("Me".to_string())
    }
}
