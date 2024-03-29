use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig, ALL_WEBSOCKET_PROTOCOLS},
    EmptyMutation, Schema,
};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    debug_handler,
    extract::{State, WebSocketUpgrade},
    http::header::HeaderMap,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};

mod auth;
mod graphql;
mod middleware;
mod models;

use auth::{Auth, TokenValidator};
use graphql::{QueryRoot, SchemaType, SubscriptionRoot};
use middleware::AuthExtension;

#[debug_handler]
async fn graphql_handler(
    State(schema): State<SchemaType>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    if let Some(token) = Auth::get_token_from_headers(&headers) {
        req = req.data(token);
    }
    schema.execute(req).await.into()
}

async fn graphql_ws_handler(
    State(schema): State<SchemaType>,
    protocol: GraphQLProtocol,
    websocket: WebSocketUpgrade,
) -> Response {
    websocket
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema.clone(), protocol)
                .on_connection_init(Auth::on_connection_init)
                .serve()
        })
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
    ))
}

#[tokio::main]
async fn main() {
    let auth_extension = AuthExtension;

    let schema = Schema::build(QueryRoot, EmptyMutation, SubscriptionRoot)
        .data(TokenValidator::new())
        .extension(auth_extension)
        .finish();

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .with_state(schema.clone())
        .route("/ws", get(graphql_ws_handler))
        .with_state(schema);

    println!("GraphQL playground: http://localhost:3000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
