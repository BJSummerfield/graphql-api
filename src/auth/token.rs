use crate::schema::User;
use async_graphql::{Error, ErrorExtensions, ServerError};

#[derive(Debug)]
pub struct Token(pub String);

pub fn is_token_valid(token: &Token) -> bool {
    token.0 == "validToken"
}

pub fn create_unauthorized_error() -> ServerError {
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

pub fn create_user_from_token(_token: &Token) -> User {
    User::new("Me".to_string())
}

pub fn authenticate<T: Into<String>>(auth_value: T) -> Result<User, ServerError> {
    let token = Token(auth_value.into());
    if is_token_valid(&token) {
        let user = create_user_from_token(&token);
        Ok(user)
    } else {
        Err(create_unauthorized_error())
    }
}
