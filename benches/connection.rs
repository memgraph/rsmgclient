use maplit::hashmap;
use rsmgclient::{ConnectParams, Connection, QueryParam};

use std::process::Command;
use std::time::Instant;
use std::{thread, time};

use serde_json::json;
use std::collections::HashMap;
use std::fs::{create_dir_all, OpenOptions};
use std::io::prelude::*;
use std::path::Path;

const NUMBER_OF_REPS: u32 = 1000;
const CONTAINER_NAME: &str = "memgraph-rsmgclient-benchmark";
const FILE_PATH: &str = "./target/benchmark-summary.json";

fn main() {
    let insert_samples = insert_query_benchmark();
    let small_query_samples = small_query_with_query_params_benchmark();
    let large_query_samples = large_query_benchmark();

    let summary = json!({
        "insert_query": {
            "samples": insert_samples,
        },
        "small_query": {
            "samples": small_query_samples
        },
        "large_query": {
            "samples": large_query_samples,
        },
    });

    write_to_file(FILE_PATH, summary.to_string().as_bytes());
}

fn start_server() -> Connection {
    // Delete container from before if present.
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("docker rm {}", CONTAINER_NAME))
        .output()
        .expect("unable to delete container");

    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("docker run -d --rm -p 7687:7687 --name {} memgraph:1.1.0-community --telemetry-enabled=False", CONTAINER_NAME))
        .output()
        .expect("failed to start server");

    // Wait until server has started.
    loop {
        match Connection::connect(&ConnectParams {
            host: Some(String::from("localhost")),
            ..Default::default()
        }) {
            Ok(connection) => {
                return connection;
            }
            Err(_) => {
                thread::sleep(time::Duration::from_millis(10));
            }
        }
    }
}

fn stop_server() {
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("docker stop {}", CONTAINER_NAME))
        .output()
        .expect("failed to stop server");
}

fn benchmark_query(
    query: &str,
    query_params: Option<&HashMap<String, QueryParam>>,
    setup: &dyn Fn(&mut Connection),
) -> Vec<f64> {
    let mut connection = start_server();

    setup(&mut connection);

    let mut samples = Vec::with_capacity(NUMBER_OF_REPS as usize);
    for _ in 0..NUMBER_OF_REPS {
        let start = Instant::now();
        let _ = match connection.execute(query, query_params) {
            Ok(cols) => cols,
            Err(err) => panic!("{}", err),
        };
        let _ = match connection.fetchall() {
            Ok(vals) => vals,
            Err(err) => panic!("{}", err),
        };
        // Convert to ms.
        samples.push(start.elapsed().as_nanos() as f64 / 1e6_f64);
    }

    stop_server();

    samples
}

fn write_to_file(file_name: &str, data: &[u8]) {
    let path = Path::new(file_name);
    match path.parent() {
        Some(p) => create_dir_all(p).expect("Unable to create dirs"),
        None => {}
    }
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .expect("unable to write to file");
    file.write_all(data).expect("unable to write to file");
}

fn insert_query_benchmark() -> Vec<f64> {
    benchmark_query(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
        None,
        &|_| {},
    )
}

fn small_query_with_query_params_benchmark() -> Vec<f64> {
    benchmark_query(
        "MATCH (u:User) WHERE u.name = $name RETURN u",
        Some(&hashmap! {
            String::from("name") => QueryParam::String(String::from("Alice")),
        }),
        &|connection| {
            connection
                .execute(
                    "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
                    None,
                )
                .expect("could not setup small query");
            connection.fetchall().expect("could not setup small query");
        },
    )
}

fn large_query_benchmark() -> Vec<f64> {
    benchmark_query("MATCH (u:User) RETURN u", None, &|connection| {
        for i in 0..1000 {
            connection.execute(
                    format!(
                        "CREATE (u:User {{name: 'Alice{}'}})-[:Likes]->(m:Software {{name: 'Memgraph{}'}})",
                        i, i,
                    )
                        .as_str(),
                    None,
                ).expect("could not setup large query");
            connection.fetchall().expect("could not setup large query");
        }
    })
}
