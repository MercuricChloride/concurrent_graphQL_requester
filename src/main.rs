use ::reqwest::blocking::Client;
use anyhow::*;
use clap::Parser;
use graphql_client::Response;
use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use serde::Serialize;
use std::sync::mpsc;
use std::thread;
use std::{fs, result::Result::Ok};
use types_for_space::ResponseData;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    command: String,

    #[clap(short, long)]
    space_id: String,
}

#[derive(GraphQLQuery, Serialize)]
#[graphql(
    schema_path = "schema.json",
    query_path = "test/query_types_for_space.graphql",
    response_derives = "Debug, Serialize, Deserialize",
    skip_serializing_none
)]
pub struct TypesForSpace;

const ENDPOINT: &str = "https://api.thegraph.com/subgraphs/name/mercuricchloride/geogenesis";

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let Args { command, space_id } = args;
    println!("space_id: {}", space_id);

    let client = Client::builder()
        .user_agent("graphql-rust/0.10.0")
        .build()?;

    match &command as &str {
        "types_for_space" => {
            let query_count = 100;

            let space_id = space_id.clone();

            let start = std::time::Instant::now();

            for i in 0..query_count {
                println!("Query #{}", i);
                query_types_for_space(&client, space_id.clone()).unwrap();
            }

            let duration = start.elapsed();

            println!("Time elapsed from single threaded query:{:?}", duration);

            let start = std::time::Instant::now();

            multi_threaded_query(space_id, 100);

            let duration = start.elapsed();

            println!("Time elapsed from multi threaded query:{:?}", duration);
        }
        _ => {
            println!("Unknown command: {}", command);
        }
    }
    Ok(())
}

fn query_types_for_space(
    client: &Client,
    space_id: String,
) -> Result<Response<ResponseData>, reqwest::Error> {
    let variables = types_for_space::Variables {
        space_id: Some(space_id),
    };
    post_graphql::<TypesForSpace, _>(&client, ENDPOINT, variables)
}

fn multi_threaded_query(space_id: String, worker_count: usize) {
    let (sender, receiver) = mpsc::channel();

    for i in 0..worker_count {
        let tx = sender.clone();

        let space_id = space_id.clone();

        thread::spawn(move || {
            // println!("Starting thread #{i}");
            let client = Client::builder()
                .user_agent("graphql-rust/0.10.0")
                .build()
                .unwrap();

            let variables = types_for_space::Variables {
                space_id: Some(space_id),
            };

            let result = post_graphql::<TypesForSpace, _>(&client, ENDPOINT, variables);

            tx.send(result).unwrap();
        });
    }

    let results = receiver.iter().take(worker_count);

    println!("Got {} results", results.count());
}
