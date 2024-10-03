use std::{collections::HashMap, ops::Deref};

use async_graphql::Positioned;
use async_graphql_parser::types::{self as Q};
use blueprint::{FieldDefinition, Graph, Index, JoinField, QueryField};
use derive_setters::Setters;

use crate::error::Error;

#[derive(Debug, Clone)]
pub enum QueryPlan<Value> {
    Parallel(Vec<QueryPlan<Value>>),
    Sequence(Vec<QueryPlan<Value>>),
    Fetch(Fetch<Value>),
    Flatten {
        select: Lens,
        plan: Box<QueryPlan<Value>>,
    },
}

#[derive(Debug, Clone, Setters)]
pub struct Fetch<Value> {
    pub arguments: Vec<Argument<Value>>,
    pub directives: Vec<Directive<Value>>,
    pub selection_set: SelectionSet<Value>,
    pub representations: Option<SelectionSet<Value>>,
    pub type_name: TypeName,
    pub service: Option<Graph>,
}

#[derive(Debug, Clone)]
pub struct VariableDefinition<Value> {
    pub name: String,
    pub type_name: TypeName,
    pub nullable: bool,
    pub directives: Vec<Directive<Value>>,
    pub default_value: Option<Value>,
}

impl QueryPlan<async_graphql_value::Value> {
    pub fn fetch(fetch: Fetch<async_graphql_value::Value>) -> Self {
        QueryPlan::Fetch(fetch)
    }
}

impl QueryPlan<async_graphql_value::Value> {
    // Tries to create a new Query Plan from a GraphQL query and a Blueprint Index.
    pub fn try_new(query: &str, operation_name: &str, blueprint: &Index) -> Result<Self, Error> {
        let doc = async_graphql_parser::parse_query(query)?;

        let operation = doc
            .operations
            .iter()
            .find_map(|op| {
                if operation_name.eq("")
                    || op
                        .0
                        .map(|name| name.to_string().eq(operation_name))
                        .unwrap_or(false)
                {
                    Some(op.1.node.clone())
                } else {
                    None
                }
            })
            .expect("TODO");

        let selection_set = SelectionSet::from(&operation.selection_set.node);

        // TODO: use them
        let _directives = extract_directives(operation.directives.clone());
        // TODO: use them
        let _variables = extract_variables(operation.variable_definitions.clone());

        let parent_definition = blueprint.get_object_type_definition("Query").unwrap();

        let plans = selection_set
            .iter()
            .map(|parent_field| -> Result<_, _> {
                let parent_field_definition = match blueprint
                    .get_field(&parent_definition.name, &parent_field.name)
                    .unwrap()
                {
                    QueryField::Field((local_field_definition, _args)) => local_field_definition,
                    QueryField::InputField(_) => panic!("TODO"),
                };

                Self::recursive_prepare(
                    blueprint,
                    parent_field_definition,
                    parent_field,
                    vec![(parent_field.name.clone(), parent_field.alias.clone())],
                )
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        Ok(Self::Parallel(plans))
    }

    fn recursive_prepare(
        blueprint: &Index,
        parent_field_definition: &FieldDefinition,
        parent_field: &Field<async_graphql_value::Value>,
        path: Vec<(String, Option<String>)>,
    ) -> Result<Vec<Self>, Error> {
        // TODO: use fields to validate
        let parent_field_type_definition = blueprint
            .get_object_type_definition(&parent_field_definition.of_type.as_type_str())
            .unwrap();

        let plans = parent_field.selections
            .iter()
            .map(
                |local_field| -> Result<(Vec<(String, Option<String>)>, Vec<Self>), Error> {
                    // TODO: use args
                    let (local_field_definition, _args) = match blueprint
                        .get_field(&parent_field_definition.of_type.as_type_str(), &local_field.name)
                        .unwrap()
                    {
                        QueryField::Field(def) => def,
                        QueryField::InputField(_) => panic!("impossible"),
                    };

                    let local_field_type_definition = blueprint.get_object_type_definition(&local_field_definition.of_type.as_type_str());

                    if let Some(local_field_type_definition) = local_field_type_definition {
                        let local_path = {
                            let mut local_path = path.clone();
                            local_path.push((local_field.name.clone(), local_field.alias.clone()));
                            local_path
                        };

                        let plans = Self::recursive_prepare(
                            blueprint,
                            local_field_definition,
                            local_field,
                            local_path.clone(),
                        )?;


                        println!("------------------------------------ [STR: INNER OPTIMIZE] ------------------------------------");
                        println!("PATH: {:?}", path);
                        println!("LOCAL PATH: {:?}", local_path);
                        println!("JF: {:?}", parent_field_definition.join_fields.iter().filter_map(|jf| jf.graph.clone()).collect::<Vec<_>>());
                        println!("LOCAL JF: {:?}", local_field_definition.join_fields.iter().filter_map(|jf| jf.graph.clone()).collect::<Vec<_>>());
                        println!("JT: {:?}", parent_field_type_definition.join_types.iter().map(|jf| jf.graph.clone()).collect::<Vec<_>>());
                        println!("LOCAL JT: {:?}", local_field_type_definition.join_types.iter().map(|jf| jf.graph.clone()).collect::<Vec<_>>());
                        println!("------------------------------------ [MID: INNER OPTIMIZE] ------------------------------------");
                        println!("{:#?}", plans);
                        println!("------------------------------------ [END: INNER OPTIMIZE] ------------------------------------");

                        Ok((local_path, plans))
                    } else {
                        // TODO: better logic
                        // TODO: handle all join__field properties (eg: external, requires, provides)
                        let target_service = {
                            let mut parent_join_fields = parent_field_definition.join_fields.iter().filter_map(|jf| jf.graph.clone()).collect::<Vec<_>>();
                            let mut local_join_fields = local_field_definition.join_fields.iter().filter_map(|jf| jf.graph.clone()).collect::<Vec<_>>();
                            if local_join_fields.len() == 1 {
                                local_join_fields.pop()
                            } else if parent_join_fields.len() == 1 {
                                parent_join_fields.pop()
                            } else {
                                None
                            }
                        };

                        let plan = Self::fetch(Fetch {
                            selection_set: SelectionSet(vec![local_field.clone()]),
                            representations: None,
                            type_name: TypeName::new(local_field_definition.of_type.as_type_str()),
                            service: target_service,

                            arguments: Vec::new(),
                            directives: Vec::new(),
                        });


                        println!("************************************ [STR: FIEND SELECT] ************************************");
                        println!("PATH: {:?}", path);
                        println!("JF: {:?}", parent_field_definition.join_fields.iter().filter_map(|jf| jf.graph.clone()).collect::<Vec<_>>());
                        println!("LOCAL JF: {:?}", local_field_definition.join_fields.iter().filter_map(|jf| jf.graph.clone()).collect::<Vec<_>>());
                        println!("JT: {:?}", parent_field_type_definition.join_types.iter().map(|jf| jf.graph.clone()).collect::<Vec<_>>());
                        println!("************************************ [MID: FIELD SELECT] ************************************");
                        println!("{:#?}", plan);
                        println!("************************************ [END: FIEND SELECT] ************************************");

                        Ok((
                            path.clone(),
                            vec![plan],
                        ))
                    }
                },
            )
            .try_fold(
                HashMap::<Vec<(String, Option<String>)>, Vec<Self>>::new(),
                |mut acc, cur| {
                    let (path, mut plans) = cur?;

                    match acc.remove(&path) {
                        Some(mut other) => {
                            plans.append(&mut other);
                            acc.insert(path, plans);
                        }
                        None => {
                            acc.insert(path, plans);
                        }
                    }

                    Ok::<
                        HashMap<
                            Vec<(std::string::String, Option<std::string::String>)>,
                            Vec<QueryPlan<async_graphql_value::Value>>,
                        >,
                        Error,
                    >(acc)
                },
            )?;

        // TODO: optimize
        println!("==================================== [STR: OUT OPTIMIZE] ====================================");
        println!("PATH: {:?}", path);
        println!("JF: {:?}", parent_field_definition.join_fields.iter().filter_map(|jf| jf.graph.clone()).collect::<Vec<_>>());
        println!("JT: {:?}", parent_field_type_definition.join_types.iter().map(|jf| jf.graph.clone()).collect::<Vec<_>>());
        println!("==================================== [MID: OUT OPTIMIZE] ====================================");
        println!("{:#?}", plans);
        println!("==================================== [END: OUT OPTIMIZE] ====================================");

        Ok(plans.values().flatten().cloned().collect())
    }
}

#[derive(Debug, Clone)]
pub struct TypeName(String);

impl TypeName {
    pub fn new(name: String) -> Self {
        TypeName(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Default, Debug, Clone)]
pub struct SelectionSet<Value>(Vec<Field<Value>>);

impl<A> Deref for SelectionSet<A> {
    type Target = Vec<Field<A>>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<Value> SelectionSet<Value> {
    pub fn new(fields: Vec<Field<Value>>) -> Self {
        Self(fields)
    }

    pub fn push(&mut self, field: Field<Value>) {
        self.0.push(field);
    }

    pub fn into_vec(self) -> Vec<Field<Value>> {
        self.0
    }
}

#[derive(Debug, Clone, Setters)]
pub struct Field<Value> {
    pub name: String,
    pub alias: Option<String>,
    pub selections: SelectionSet<Value>,
    pub arguments: Vec<Argument<Value>>,
    pub directives: Vec<Directive<Value>>,

    /// When set to true the field is considered to be used internally for
    /// querying sub-graphs and should not be exposed to the user.
    pub is_hidden: bool,

    /// Possible Graphs from where the field can be queried from.
    pub graph: Vec<Graph>,

    /// Internal readonly information from the Blueprint Index.
    pub join_field: Vec<JoinField>,

    /// The type of the field.
    pub field_type: Option<TypeName>,

    /// The type in which this field is defined.
    pub parent_type: Option<TypeName>,
}

impl<A> Field<A> {
    pub fn new(name: String, selections: SelectionSet<A>) -> Self {
        Field {
            name: name.to_string(),
            alias: None,
            selections,
            arguments: Vec::new(),
            directives: Vec::new(),
            is_hidden: false,
            graph: Vec::new(),
            join_field: Vec::new(),
            parent_type: None,
            field_type: None,
        }
    }
}

#[derive(Debug, Clone, Setters)]
pub struct Argument<Value> {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, Setters)]
pub struct Directive<Value> {
    pub name: String,
    pub arguments: Vec<Argument<Value>>,
}

#[derive(Debug, Clone)]
pub enum Lens {
    Field(String),
    Index(usize),
    Combine(Box<Lens>, Box<Lens>),
    ForEach(Box<Lens>),
    Empty,
}

impl Lens {
    pub fn get(&self, value: serde_json::Value) -> serde_json::Value {
        match self {
            Lens::Field(key) => match value {
                serde_json::Value::Object(obj) => {
                    obj.get(key).cloned().unwrap_or(serde_json::Value::Null)
                }
                _ => serde_json::Value::Null,
            },
            Lens::Index(index) => match value {
                serde_json::Value::Array(vec) => {
                    vec.get(*index).cloned().unwrap_or(serde_json::Value::Null)
                }
                _ => serde_json::Value::Null,
            },
            Lens::Combine(first_lens, second_lens) => {
                let value = first_lens.get(value);
                second_lens.get(value)
            }
            Lens::ForEach(local_lens) => match value {
                serde_json::Value::Array(vec) => serde_json::Value::Array(
                    vec.into_iter().map(|value| local_lens.get(value)).collect(),
                ),
                serde_json::Value::Object(map) => serde_json::Value::Object(
                    map.into_iter()
                        .map(|(key, value)| (key, local_lens.get(value)))
                        .collect(),
                ),
                _ => serde_json::Value::Null,
            },
            Lens::Empty => serde_json::Value::Null,
        }
    }

    pub fn set(
        &self,
        value: serde_json::Value,
        other_value: serde_json::Value,
    ) -> serde_json::Value {
        match self {
            Lens::Field(key) => match value {
                serde_json::Value::Object(mut obj) => {
                    obj.insert(key.clone(), other_value);
                    serde_json::Value::Object(obj)
                }
                _ => serde_json::json!({ key: other_value }),
            },
            Lens::Index(index) => match value {
                serde_json::Value::Array(mut vec) => {
                    if index >= &vec.len() {
                        vec.resize_with(index + 1, || serde_json::Value::Null);
                    }
                    vec[*index] = other_value;
                    serde_json::Value::Array(vec)
                }
                _ => serde_json::json!([other_value]),
            },
            Lens::Combine(first_lens, second_lens) => {
                let intermediate = first_lens.get(value.clone());
                let second_value = second_lens.set(intermediate, other_value);
                first_lens.set(value, second_value)
            }
            Lens::ForEach(local_lens) => match value {
                serde_json::Value::Array(vec) => serde_json::Value::Array(
                    vec.into_iter()
                        .map(|v| local_lens.set(v, other_value.clone()))
                        .collect(),
                ),
                serde_json::Value::Object(map) => serde_json::Value::Object(
                    map.into_iter()
                        .map(|(key, v)| (key, local_lens.set(v, other_value.clone())))
                        .collect(),
                ),
                _ => other_value,
            },
            Lens::Empty => other_value,
        }
    }
}

// Correctly implement and add tests
impl From<&Q::SelectionSet> for SelectionSet<async_graphql_value::Value> {
    fn from(node: &Q::SelectionSet) -> SelectionSet<async_graphql_value::Value> {
        let mut selection_set = Vec::new();
        for selection in node.items.iter() {
            let inner_selection = &selection.node;
            match inner_selection {
                Q::Selection::Field(Positioned { node, .. }) => {
                    let field_name = node.name.node.as_str().to_string();

                    let alias = node
                        .alias
                        .as_ref()
                        .map(|alias| alias.clone().into_inner().to_string());

                    let arguments = extract_arguments(node.arguments.clone());

                    let directives = extract_directives(node.directives.clone());

                    let field =
                        Field::new(field_name, SelectionSet::from(&node.selection_set.node))
                            .alias(alias)
                            .arguments(arguments)
                            .directives(directives);

                    selection_set.push(field);
                }
                Q::Selection::InlineFragment(_) => {
                    todo!()
                }
                Q::Selection::FragmentSpread(_) => {
                    todo!()
                }
            }
        }
        SelectionSet(selection_set)
    }
}

fn extract_directives(
    directives: Vec<Positioned<Q::Directive>>,
) -> Vec<Directive<async_graphql_value::Value>> {
    directives
        .into_iter()
        .map(|Positioned { node: dir_node, .. }| {
            let arguments = extract_arguments(dir_node.arguments);

            Directive { name: dir_node.name.into_inner().to_string(), arguments }
        })
        .collect()
}

fn extract_arguments(
    arguments: Vec<(
        Positioned<async_graphql_value::Name>,
        Positioned<async_graphql_value::Value>,
    )>,
) -> Vec<Argument<async_graphql_value::Value>> {
    arguments
        .into_iter()
        .map(
            |(Positioned { node: name_node, .. }, Positioned { node: arg_node, .. })| Argument {
                name: name_node.to_string(),
                value: arg_node,
            },
        )
        .collect()
}

fn extract_variables(
    variable_definitions: Vec<Positioned<Q::VariableDefinition>>,
) -> Vec<VariableDefinition<async_graphql_value::Value>> {
    variable_definitions
        .into_iter()
        .map(
            |Positioned { node: variable_node, .. }| VariableDefinition {
                name: variable_node.name.node.to_string(),
                type_name: TypeName::new(variable_node.var_type.node.base.to_string()),
                nullable: variable_node.var_type.node.nullable,
                directives: extract_directives(variable_node.directives.clone()),
                default_value: variable_node
                    .default_value()
                    .cloned()
                    .map(|cv| cv.into_value()),
            },
        )
        .collect()
}

#[cfg(test)]
mod test {
    use insta::assert_debug_snapshot;

    use crate::QueryPlan;

    #[test]
    fn test() {
        let schema = resource::resource_str!("../../examples/router.graphql");
        let blueprint = blueprint::Blueprint::parse(&schema).unwrap();
        let query = r#"
            query {
                history: me {
                    id
                    name
                    photo {
                        url
                    }
                    orders {
                        id
                        quantity
                        product {
                            id
                            name
                            price
                        }
                    }
                }
            }
        "#;
        assert_debug_snapshot!(blueprint.to_index());
        let actual: QueryPlan<_> = QueryPlan::try_new(query, "", &blueprint.to_index()).unwrap();
        assert_debug_snapshot!(actual);
    }
}
