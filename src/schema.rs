use async_graphql::*;

#[derive(Debug)]
pub struct Token(pub String);

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

    #[graphql(guard = "check_auth()")]
    async fn welcome_message(&self, ctx: &Context<'_>) -> Result<String> {
        let user = ctx.data_unchecked::<User>();
        Ok(format!("Welcome, {}!", user.name))
    }
}

fn check_auth() -> impl Guard {
    |ctx: &Context<'_>| {
        if ctx.data_opt::<User>().is_none() {
            return Err(Error::new("Unauthorized")
                .extend_with(|_, e| e.set("status", 401))
                .extend_with(|_, e| e.set("code", "UNAUTHORIZED")));
        }
        Ok(())
    }
}

pub type Schema = async_graphql::Schema<QueryRoot, EmptyMutation, EmptySubscription>;
