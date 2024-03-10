use async_graphql::*;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn my_query(&self) -> &str {
        "Hello, world!"
    }
}

pub type Schema = async_graphql::Schema<QueryRoot, EmptyMutation, EmptySubscription>;
