use crate::models::User;
use async_graphql::{Context, Object, Result};

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
