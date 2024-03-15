use crate::models::User;
use async_graphql::{Context, Object, Result};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn welcome_message(&self, ctx: &Context<'_>) -> Result<String> {
        let user = ctx.data_unchecked::<User>();
        Ok(format!("Welcome, {}!", user.upn))
    }

    async fn get_user<'ctx>(&self, ctx: &'ctx Context<'_>) -> Result<Option<&'ctx User>> {
        Ok(ctx.data_opt::<User>())
    }
}
