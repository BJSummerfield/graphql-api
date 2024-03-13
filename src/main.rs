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

use auth::{Auth, TokenValidator};
use graphql::{QueryRoot, SchemaType, SubscriptionRoot};

#[derive(Clone)]
struct AppState {
    schema: SchemaType,
    auth: Auth,
}

async fn graphql_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .unwrap_or_default();

    app_state
        .auth
        .process_request_with_token(app_state.schema.clone(), req, token)
        .await
}

async fn graphql_ws_handler(
    State(app_state): State<AppState>,
    protocol: GraphQLProtocol,
    websocket: WebSocketUpgrade,
) -> Response {
    websocket
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, app_state.schema.clone(), protocol)
                .on_connection_init(move |value| app_state.auth.on_connection_init(value))
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
    let auth = Auth::new(TokenValidator::new());
    let schema = Schema::build(QueryRoot, EmptyMutation, SubscriptionRoot).finish();

    let app_state = AppState { schema, auth };

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .route("/ws", get(graphql_ws_handler))
        .with_state(app_state);

    println!("GraphQL playground: http://localhost:3000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
