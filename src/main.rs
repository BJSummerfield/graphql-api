use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig, ALL_WEBSOCKET_PROTOCOLS},
    EmptyMutation, Schema,
};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    http::HeaderMap,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};

mod auth;
mod graphql;
mod models;

use auth::{authenticate, create_unauthorized_error};
use graphql::{on_connection_init, QueryRoot, SchemaType, SubscriptionRoot};

async fn graphql_handler(
    State(schema): State<SchemaType>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    if let Some(auth_header) = headers.get("Authorization") {
        match authenticate(auth_header.to_str().unwrap()) {
            Ok(user) => {
                println!("User: {:?}", user);
                req = req.data(user);
                schema.execute(req).await.into()
            }
            Err(error) => {
                let response = async_graphql::Response::from_errors(vec![error]);
                GraphQLResponse::from(response)
            }
        }
    } else {
        let response = async_graphql::Response::from_errors(vec![create_unauthorized_error()]);
        GraphQLResponse::from(response)
    }
}

async fn graphql_ws_handler(
    State(schema): State<SchemaType>,
    protocol: GraphQLProtocol,
    websocket: WebSocketUpgrade,
) -> Response {
    println!("Websocket connection initiated!");
    websocket
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema.clone(), protocol)
                .on_connection_init(on_connection_init)
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
    let schema = Schema::build(QueryRoot, EmptyMutation, SubscriptionRoot).finish();

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route("/ws", get(graphql_ws_handler))
        .with_state(schema);

    println!("GraphQL playground: http://localhost:3000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
