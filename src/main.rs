use ::reqwest::blocking::Client;
use anyhow::*;
use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "test/query_2.graphql",
    response_derives = "Debug"
)]
pub struct TypesForSpace;

fn main() -> Result<(), anyhow::Error> {
    let endpoint = "https://api.thegraph.com/subgraphs/name/mercuricchloride/geogenesis";
    let space_id = std::env::args().nth(1).unwrap();
    println!("space_id: {}", space_id);

    let client = Client::builder()
        .user_agent("graphql-rust/0.10.0")
        .build()?;

    let variables = types_for_space::Variables {
        space_id: Some(space_id),
    };

    let response_body = post_graphql::<TypesForSpace, _>(&client, endpoint, variables).unwrap();

    println!("{:#?}", response_body);

    Ok(())
}
