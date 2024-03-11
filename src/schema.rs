use async_graphql::{Context, Data, EmptyMutation, Object, Result, Subscription};
use futures_util::Stream;
use serde::Deserialize;

#[derive(Debug)]
pub struct Token(pub String);

#[derive(Debug)]
pub struct User {
    pub name: String,
}

impl User {
    pub fn new(name: String) -> Self {
        User { name }
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn current_user<'a>(&self, ctx: &'a Context<'_>) -> Option<&'a str> {
        ctx.data_opt::<User>().map(|user| user.name.as_str())
    }

    async fn welcome_message(&self, ctx: &Context<'_>) -> Result<String> {
        let user = ctx.data_unchecked::<User>();
        Ok(format!("Welcome, {}!", user.name))
    }
}

pub type SchemaType = async_graphql::Schema<QueryRoot, EmptyMutation, SubscriptionRoot>;

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn values(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = i32>> {
        if ctx.data::<Token>()?.0 != "validToken" {
            return Err("Forbidden".into());
        }
        Ok(futures_util::stream::once(async move { 10 }))
    }
}

pub async fn on_connection_init(value: serde_json::Value) -> Result<Data> {
    #[derive(Deserialize)]
    struct Payload {
        #[serde(rename = "Authorization")]
        authorization: String,
    }

    if let Ok(payload) = serde_json::from_value::<Payload>(value) {
        let mut data = Data::default();
        data.insert(Token(payload.authorization));
        Ok(data)
    } else {
        Err("Token is required".into())
    }
}
