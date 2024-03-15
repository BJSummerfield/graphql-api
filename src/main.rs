use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig, ALL_WEBSOCKET_PROTOCOLS},
    EmptyMutation, EmptySubscription, Schema,
};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    debug_handler,
    extract::{State, WebSocketUpgrade},
    http::header::HeaderMap,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};

mod auth;
mod graphql;
mod middleware;
mod models;

use auth::Auth;
use graphql::{QueryRoot, SchemaType, SubscriptionRoot};
use middleware::{AuthExtension, Token};

#[debug_handler]
async fn graphql_handler(
    State(schema): State<SchemaType>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    if let Some(token) = get_token_from_headers(&headers) {
        println!("Token: {:?}", token.0);
        req = req.data(token);
    }
    schema.execute(req).await.into()
}

fn get_token_from_headers(headers: &HeaderMap) -> Option<Token> {
    println!("get_token_from_headers");
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().map(|s| Token(s.to_string())).ok())
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
    Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

#[tokio::main]
async fn main() {
    println!("Starting server...");
    let auth_extension = AuthExtension;
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .extension(auth_extension)
        .finish();

    // let auth_middleware_layer = ServiceBuilder::new().layer_fn(|inner| AuthMiddleware::new(inner));

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .with_state(schema); // .route("/ws", get(graphql_ws_handler))

    println!("GraphQL playground: http://localhost:3000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
