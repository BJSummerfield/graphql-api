mod schema;

use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptyMutation, EmptySubscription,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::HeaderMap,
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use schema::{QueryRoot, Schema, Token, User};

// async fn graphql_handler(schema: Extension<Schema>, req: GraphQLRequest) -> GraphQLResponse {
//     schema.execute(req.into_inner()).await.into()
// }

async fn graphql_handler(
    State(schema): State<Schema>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    if let Some(token) = get_token_from_headers(&headers) {
        println!("Token: {:?}", token);
        let user = User::new("Me".to_string());
        req = req.data(user);
    }
    schema.execute(req).await.into()
}
fn get_token_from_headers(headers: &HeaderMap) -> Option<Token> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().map(|s| Token(s.to_string())).ok())
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

    // println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    // Run the server
    axum::serve(listener, app).await.unwrap();
}
