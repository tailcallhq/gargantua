use derive_more::From;

#[derive(From)]
pub enum Error {
    Parse(async_graphql_parser::Error),
}
