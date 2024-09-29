use derive_more::From;

#[derive(From, Debug)]
pub enum Error {
    Blueprint(blueprint::error::Error),
    Parse(async_graphql_parser::Error),

    // Error while creating the query plan
    Plan(valid::Error<String>),
}
