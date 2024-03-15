use crate::auth::Token;
use crate::models::User;
use async_graphql::{Context, Result, Subscription};
use futures_util::stream::{once, Stream};

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn current_user<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
    ) -> Result<impl Stream<Item = String> + 'ctx> {
        let user = ctx.data::<User>()?;
        Ok(once(async move { user.upn.clone() }))
    }

    async fn current_token<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
    ) -> Result<impl Stream<Item = String> + 'ctx> {
        let token = ctx.data::<Token>()?;
        Ok(once(async move { token.0.clone() }))
    }

    async fn get_user<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
    ) -> Result<impl Stream<Item = User> + 'ctx> {
        let user = ctx.data::<User>()?;
        Ok(once(async move { user.clone() }))
    }
}
