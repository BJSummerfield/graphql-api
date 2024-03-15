use crate::auth::Token;
use crate::models::User;

use async_graphql::{
    extensions::{Extension, ExtensionContext, ExtensionFactory, NextPrepareRequest},
    Error, ErrorExtensions, Request, ServerError, ServerResult,
};
use std::sync::Arc;

pub struct AuthExtension;

#[async_trait::async_trait]
impl Extension for AuthExtension {
    async fn prepare_request(
        &self,
        ctx: &ExtensionContext<'_>,
        mut request: Request,
        next: NextPrepareRequest<'_>,
    ) -> ServerResult<Request> {
        if let Some(token_str) = ctx.data_opt::<Token>().map(|token| &token.0) {
            if token_str == "validToken" {
                let token = Token(token_str.clone());
                let user = User {
                    id: "1".to_string(),
                    upn: "billy@yahoo.com".to_string(),
                };

                request = request.data(token).data(user);

                next.run(ctx, request).await
            } else {
                println!("Invalid token");
                Err(create_unauthorized_error("Invalid Token"))
            }
        } else {
            println!("No token provided");
            Err(create_unauthorized_error("No Token Provided"))
        }
    }
}

impl ExtensionFactory for AuthExtension {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(AuthExtension {})
    }
}

fn create_unauthorized_error(message: &str) -> ServerError {
    let error = Error::new(message)
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
