#[derive(Debug, Clone)]
pub enum QueryPlan {
    Parallel(Vec<QueryPlan>),
    Sequence(Vec<QueryPlan>),
    Fetch {
        service: GraphId,
        query: String,
    },
    Flatten {
        path: String,
        plan: Box<QueryPlan>,
    },
}