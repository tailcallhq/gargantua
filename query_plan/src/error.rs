use derive_more::From;
use valid::ValidationError;

#[derive(From, Debug)]
pub enum Error {
    Blueprint(blueprint::error::Error),
    Parse(async_graphql_parser::Error),

    // Error while creating the query plan
    Plan(ValidationError<String>),
}
