use crate::context::Data;
use crate::model::__DirectiveLocation;
use crate::query::QueryBuilder;
use crate::registry::{Directive, InputValue, Registry};
use crate::types::QueryRoot;
use crate::{ObjectType, SubscribeBuilder, SubscriptionType, Type};
use std::any::Any;
use std::collections::HashMap;

/// GraphQL schema
pub struct Schema<Query, Mutation, Subscription> {
    query: QueryRoot<Query>,
    mutation: Mutation,
    pub(crate) subscription: Subscription,
    pub(crate) registry: Registry,
    pub(crate) data: Data,
}

impl<Query: ObjectType, Mutation: ObjectType, Subscription: SubscriptionType>
    Schema<Query, Mutation, Subscription>
{
    /// Create a schema
    ///
    /// The root object for the query and Mutation needs to be specified.
    /// If there is no mutation, you can use `EmptyMutation`.
    /// If there is no subscription, you can use `EmptySubscription`.
    pub fn new(query: Query, mutation: Mutation, subscription: Subscription) -> Self {
        let mut registry = Registry {
            types: Default::default(),
            directives: Default::default(),
            implements: Default::default(),
            query_type: Query::type_name().to_string(),
            mutation_type: if Mutation::is_empty() {
                None
            } else {
                Some(Mutation::type_name().to_string())
            },
            subscription_type: if Subscription::is_empty() {
                None
            } else {
                Some(Subscription::type_name().to_string())
            },
        };

        registry.add_directive(Directive {
            name: "include",
            description: Some("Directs the executor to include this field or fragment only when the `if` argument is true."),
            locations: vec![
                __DirectiveLocation::FIELD,
                __DirectiveLocation::FRAGMENT_SPREAD,
                __DirectiveLocation::INLINE_FRAGMENT
            ],
            args: {
                let mut args = HashMap::new();
                args.insert("if", InputValue {
                    name: "if",
                    description: Some("Included when true."),
                    ty: "Boolean!".to_string(),
                    default_value: None
                });
                args
            }
        });

        registry.add_directive(Directive {
            name: "skip",
            description: Some("Directs the executor to skip this field or fragment when the `if` argument is true."),
            locations: vec![
                __DirectiveLocation::FIELD,
                __DirectiveLocation::FRAGMENT_SPREAD,
                __DirectiveLocation::INLINE_FRAGMENT
            ],
            args: {
                let mut args = HashMap::new();
                args.insert("if", InputValue {
                    name: "if",
                    description: Some("Skipped when true."),
                    ty: "Boolean!".to_string(),
                    default_value: None
                });
                args
            }
        });

        // register scalars
        bool::create_type_info(&mut registry);
        i32::create_type_info(&mut registry);
        f32::create_type_info(&mut registry);
        String::create_type_info(&mut registry);

        QueryRoot::<Query>::create_type_info(&mut registry);
        if !Mutation::is_empty() {
            Mutation::create_type_info(&mut registry);
        }
        if !Subscription::is_empty() {
            Subscription::create_type_info(&mut registry);
        }

        Self {
            query: QueryRoot { inner: query },
            mutation,
            subscription,
            registry,
            data: Default::default(),
        }
    }

    /// Add a global data that can be accessed in the `Context`.
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        self.data.insert(data);
        self
    }

    /// Start a query and return `QueryBuilder`.
    pub fn query<'a>(&'a self, source: &'a str) -> QueryBuilder<'a, Query, Mutation> {
        QueryBuilder {
            query: &self.query,
            mutation: &self.mutation,
            registry: &self.registry,
            source,
            operation_name: None,
            variables: None,
            data: &self.data,
        }
    }

    /// Start a subscribe and return `SubscribeBuilder`.
    pub fn subscribe<'a>(&'a self, source: &'a str) -> SubscribeBuilder<'a, Subscription> {
        SubscribeBuilder {
            subscription: &self.subscription,
            registry: &self.registry,
            source,
            operation_name: None,
            variables: None,
        }
    }
}
