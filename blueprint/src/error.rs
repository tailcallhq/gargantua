use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    Parse(async_graphql_parser::Error),
}
