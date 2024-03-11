use crate::authenticate;
use crate::models::User;
use async_graphql::Data;
use async_graphql::{Context, Result, Subscription};
use futures_util::Stream;
use serde::Deserialize;

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn current_user<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
    ) -> Result<impl Stream<Item = String> + 'ctx> {
        let user = ctx.data::<User>()?;
        Ok(futures_util::stream::once(async move { user.name.clone() }))
    }
}

pub async fn on_connection_init(value: serde_json::Value) -> Result<Data> {
    #[derive(Deserialize)]
    struct Payload {
        #[serde(rename = "Authorization")]
        authorization: String,
    }

    if let Ok(payload) = serde_json::from_value::<Payload>(value) {
        match authenticate(payload.authorization) {
            Ok(user) => {
                let mut data = Data::default();
                data.insert(user);
                Ok(data)
            }
            Err(_) => Err("Unauthorized".into()),
        }
    } else {
        Err("Token is required".into())
    }
}
