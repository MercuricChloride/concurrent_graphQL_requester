use ::reqwest::blocking::Client;
use anyhow::*;
use clap::Parser;
use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use serde::Serialize;
use std::result::Result::Ok;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;

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
    response_derives = "Debug, Serialize, Deserialize, Clone",
    skip_serializing_none
)]
pub struct TypesForSpace;

pub const ENDPOINT: &str = "https://api.thegraph.com/subgraphs/name/mercuricchloride/geogenesis";

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let Args { command, space_id } = args;

    let space_id = Arc::new(RwLock::new(space_id));
    let client = Arc::new(RwLock::new(Client::new()));

    match &command as &str {
        "types_for_space" => {
            let query_count = 10;

            let space_id = space_id.clone();

            // let start = std::time::Instant::now();

            // for i in 0..query_count {
            //     println!("Query #{}", i);
            //     query_types_for_space(&client, &space_id).unwrap();
            // }

            // let duration = start.elapsed();

            // println!("Time elapsed from single threaded query:{:?}", duration);

            let start = std::time::Instant::now();
            multi_threaded_query(space_id, client, query_count);
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
    client: &Arc<RwLock<Client>>,
    space_id: &Arc<RwLock<String>>,
) -> Result<(), reqwest::Error> {
    let variables = types_for_space::Variables {
        space_id: Some(space_id.read().unwrap().clone()),
    };
    post_graphql::<TypesForSpace, _>(&client.read().unwrap().clone(), ENDPOINT, variables)?;
    Ok(())
}

pub fn multi_threaded_query(
    space_id: Arc<RwLock<String>>,
    client: Arc<RwLock<Client>>,
    worker_count: usize,
) {
    let (sender, receiver) = mpsc::channel();

    for _ in 0..worker_count {
        let tx = sender.clone();

        let client = client.read().unwrap().clone();

        let space_id = space_id.read().unwrap().clone();

        let variables = types_for_space::Variables {
            space_id: Some(space_id),
        };

        thread::spawn(move || {
            let result = post_graphql::<TypesForSpace, _>(&client, ENDPOINT, variables).unwrap();
            tx.send(result).unwrap();
        });
    }

    drop(sender);

    println!("Got {} results", receiver.iter().count());
}
