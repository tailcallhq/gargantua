use async_graphql_parser::types::ServiceDocument;
use valid::Valid;

use crate::Blueprint;

pub fn parse(doc: ServiceDocument) -> Valid<Blueprint, String> {
    // @karatakis I'm not sure what to do here?
    todo!()
}
