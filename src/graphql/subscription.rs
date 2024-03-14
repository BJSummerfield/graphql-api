use crate::models::User;
use async_graphql::{Context, Result, Subscription};
use futures_util::Stream;

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn current_user<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
    ) -> Result<impl Stream<Item = String> + 'ctx> {
        let user = ctx.data::<User>()?;
        Ok(futures_util::stream::once(async move { user.upn.clone() }))
    }
}
