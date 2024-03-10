use async_graphql::*;

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

pub type Schema = async_graphql::Schema<QueryRoot, EmptyMutation, EmptySubscription>;
