mod blueprint;
mod build;

pub use blueprint::*;
pub use build::parse;
pub use {async_graphql_parser, async_graphql_value, url};

#[cfg(test)]
mod tests {
    use valid::Validator;

    use super::*;

    #[test]
    fn test_parse() {
        let graphql = resource::resource_str!("../examples/router.graphql");
        let document = async_graphql_parser::parse_schema(graphql).unwrap();
        let blueprint = parse(document)
            .map(|b| serde_json::to_string_pretty(&b).unwrap())
            .to_result()
            .unwrap();
        insta::assert_snapshot!(blueprint);
    }
}
