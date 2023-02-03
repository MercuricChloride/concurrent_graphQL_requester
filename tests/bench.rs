#![feature(test)]

extern crate test;

use std::{
    sync::{mpsc, Arc, RwLock},
    thread,
};

use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;
use serde::Serialize;
use test::Bencher;

#[derive(GraphQLQuery, Serialize)]
#[graphql(
    schema_path = "schema.json",
    query_path = "test/query_types_for_space.graphql",
    response_derives = "Debug, Serialize, Deserialize, Clone",
    skip_serializing_none
)]
pub struct TypesForSpace;

pub const ENDPOINT: &str = "https://api.thegraph.com/subgraphs/name/mercuricchloride/geogenesis";

#[bench]
fn bench_channels(b: &mut Bencher) {
    b.iter(|| {
        let space_id = Arc::new(RwLock::new(String::from(
            "0xe3d08763498e3247EC00A481F199B018f2148723",
        )));
        let client = Arc::new(RwLock::new(Client::new()));

        let query_count = 10;
        multi_threaded_query(space_id, client, query_count);
    })
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
