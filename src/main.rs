mod auth;
mod schema;

use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptyMutation, EmptySubscription, Error, ErrorExtensions, Response, ServerError,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::HeaderMap,
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use schema::{QueryRoot, Schema};

use auth::authenticate;

async fn graphql_handler(
    State(schema): State<Schema>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    match authenticate(&headers) {
        Ok(user) => {
            println!("User: {:?}", user);
            req = req.data(user);
            schema.execute(req).await.into()
        }
        Err(error) => {
            let response = Response::from_errors(vec![error]);
            GraphQLResponse::from(response)
        }
    }
}

async fn graphql_playground() -> impl IntoResponse {
    response::Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

#[tokio::main]
async fn main() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .with_state(schema);

    println!("GraphQL playground: http://localhost:3000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
