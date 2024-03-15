use crate::models::User;

use async_graphql::{
    extensions::{Extension, ExtensionContext, ExtensionFactory, NextPrepareRequest},
    Error, ErrorExtensions, Request, ServerError, ServerResult,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Token(pub String);
pub struct AuthExtension;

#[async_trait::async_trait]
impl Extension for AuthExtension {
    async fn prepare_request(
        &self,
        ctx: &ExtensionContext<'_>,
        mut request: Request,
        next: NextPrepareRequest<'_>,
    ) -> ServerResult<Request> {
        println!("AuthExtension::prepare_request");
        // Here, extract the token from HTTP headers, which should have been added to the request data
        // For the purpose of this example, let's assume the token is directly available
        // This part might need to be adjusted based on your actual token extraction logic
        // let user = ctx.data_unchecked::<User>();
        println!(
            "{:?}",
            ctx.data_opt::<Token>().map(|token| token.0.as_str())
        );
        if let Some(token_str) = ctx.data_opt::<Token>().map(|token| token.0.clone()) {
            // Validate the token and create a user object
            // Here, insert your token validation logic
            if token_str == "validToken" {
                let token = Token(token_str.clone()); // Create your Token struct
                let user = User {
                    id: "1".to_string(),
                    upn: "billy@yahoo.com".to_string(),
                };

                // Modify the request to include both the Token and User
                request = request.data(token).data(user);

                // Proceed with the modified request
                next.run(ctx, request).await
            } else {
                println!("Invalid token");
                // If the token is invalid, return an error.
                Err(create_unauthorized_error("Unauthorized"))
            }
        } else {
            println!("No token provided");
            // If no token is provided, return an error.
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
