use super::{QueryRoot, SubscriptionRoot};
use async_graphql::{EmptyMutation, Schema};

pub type SchemaType = Schema<QueryRoot, EmptyMutation, SubscriptionRoot>;
