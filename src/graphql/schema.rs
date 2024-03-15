use super::{QueryRoot, SubscriptionRoot};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};

pub type SchemaType = Schema<QueryRoot, EmptyMutation, EmptySubscription>;
