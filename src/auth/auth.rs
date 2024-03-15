use crate::auth::TokenValidator;
use crate::graphql::SchemaType;
use crate::models::User;
use async_graphql::{Data, Error, ErrorExtensions, Result, ServerError};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, http::HeaderMap};
use serde::Deserialize;

pub struct Token(pub String);

#[derive(Deserialize)]
struct Payload {
    #[serde(rename = "Authorization")]
    authorization: String,
}

#[derive(Clone)]
pub struct Auth;
// pub token_validator: TokenValidator,

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
// pub fn new(token_validator: TokenValidator) -> Self {
//     Self { token_validator }
// }

// pub async fn http(
//     &self,
//     State(schema): State<SchemaType>,
//     req: GraphQLRequest,
//     headers: HeaderMap,
// ) -> GraphQLResponse {
//     match Self::extract_token(&headers) {
//         Some(token) => self.process_request_with_token(schema, req, token).await,
//         None => Self::unauthorized_response(),
//     }
// }
//
// fn extract_token(headers: &HeaderMap) -> Option<String> {
//     headers
//         .get("Authorization")
//         .map(|value| value.to_str().unwrap().to_string())
// }
//
// pub async fn process_request_with_token(
//     &self,
//     schema: SchemaType,
//     req: GraphQLRequest,
//     token: String,
// ) -> GraphQLResponse {
//     match self.token_validator.validate_token(&token).await {
//         Ok(user) => {
//             let req = req.into_inner().data(user);
//             schema.execute(req).await.into()
//         }
//         // Err(error) => GraphQLResponse::from(async_graphql::Response::from_errors(vec![error])),
//         Err(error) => {
//             println!("Error: {:?}", error);
//             GraphQLResponse::from(async_graphql::Response::from_errors(vec![
//                 Self::create_unauthorized_error(),
//             ]))
//         }
//     }
// }
//
// fn unauthorized_response() -> GraphQLResponse {
//     let response =
//         async_graphql::Response::from_errors(vec![Self::create_unauthorized_error()]);
//     GraphQLResponse::from(response)
// }
//
// fn authenticate(auth_value: String) -> Result<User, ServerError> {
//     let token = auth_value.into();
//     if Self::is_token_valid(&token) {
//         let user = Self::create_user_from_token(&token);
//         Ok(user)
//     } else {
//         Err(Self::create_unauthorized_error())
//     }
// }
//
// fn is_token_valid(token: &String) -> bool {
//     token == "validToken"
// }

// fn create_unauthorized_error() -> ServerError {
//     let error = Error::new("Unauthorized")
//         .extend_with(|_, e| e.set("status", 401))
//         .extend_with(|_, e| e.set("code", "UNAUTHORIZED"));
//     ServerError {
//         message: error.message,
//         source: error.source,
//         locations: Vec::new(),
//         path: Vec::new(),
//         extensions: error.extensions,
//     }
// }
//
// fn create_user_from_token(_token: &String) -> User {
//     User::new("Me".to_string())

// }
