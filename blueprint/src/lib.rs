mod blueprint;
mod parse;

pub use url;
pub use async_graphql_parser;
pub use async_graphql_value;

pub use blueprint::*;
pub use parse::parse;

#[cfg(test)]
mod tests {
    use super::*;
    use valid::Validator;

    #[test]
    fn test_parse() {
        let graphql = resource::resource_str!("../examples/router.graphql");
        let document = async_graphql_parser::parse_schema(graphql).unwrap();
        let blueprint = parse(document).map(|b| serde_json::to_string_pretty(&b).unwrap()).to_result().unwrap();
        insta::assert_snapshot!(blueprint);
    }
}
