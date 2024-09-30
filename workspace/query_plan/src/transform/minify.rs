use std::marker::PhantomData;

use valid::{Transform, Valid, Validator};

use crate::error::Error;
use crate::QueryPlan;
pub struct Minify<A>(PhantomData<A>);

impl<A> Minify<A> {
    pub fn new() -> Self {
        Minify(PhantomData)
    }
}

impl<A> Transform for Minify<A> {
    type Value = QueryPlan<A>;
    type Error = Error;

    fn transform(&self, input: Self::Value) -> Valid<Self::Value, Self::Error> {
        match input {
            QueryPlan::Parallel(items) => {
                if items.len() == 1 {
                    match items.into_iter().next() {
                        Some(item) => self.transform(item),
                        None => Valid::fail(Error::Plan(valid::Error::new(
                            "Empty Parallel".to_string(),
                        ))),
                    }
                } else {
                    Valid::from_iter(items.into_iter(), |item| self.transform(item))
                        .map(|items| QueryPlan::Parallel(items))
                }
            }
            QueryPlan::Sequence(vec) => {
                if vec.len() == 1 {
                    match vec.into_iter().next() {
                        Some(item) => self.transform(item),
                        None => Valid::fail(Error::Plan(valid::Error::new(
                            "Empty Sequence".to_string(),
                        ))),
                    }
                } else {
                    Valid::from_iter(vec.into_iter(), |item| self.transform(item))
                        .map(|vec| QueryPlan::Sequence(vec))
                }
            }
            QueryPlan::Fetch { .. } => Valid::succeed(input),
            QueryPlan::Flatten { select, plan } => self
                .transform(*plan)
                .map(|plan| QueryPlan::Flatten { select, plan: Box::new(plan) }),
        }
    }
}
