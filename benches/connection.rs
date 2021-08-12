use maplit::hashmap;
use rsmgclient::{ConnectParams, Connection, MgError, QueryParam};

use std::process::Command;
use std::time::Instant;
use std::{thread, time};

use serde_json::json;
use std::collections::HashMap;
use std::fs::{create_dir_all, OpenOptions};
use std::io::prelude::*;
use std::path::Path;

const NUMBER_OF_REPS: u32 = 100;
const CONTAINER_NAME: &str = "memgraph-rsmgclient-benchmark";
const FILE_PATH: &str = "./target/benchmark-summary.json";
const MEMGRAPH_VERSION: &str = "memgraph:1.6.0-community";

fn main() {
    let insert_samples = insert_query_benchmark();
    let small_query_samples = small_query_with_query_params_benchmark();
    let small_query_2_samples = small_query_with_query_params_2_benchmark();
    let large_query_samples = large_query_benchmark();
    let large_query_2_samples = large_query_2_benchmark();

    let summary = json!({
        "insert_query": {
            "samples": insert_samples,
        },
        "small_query": {
            "samples": small_query_samples,
        },
        "small_query_2": {
            "samples": small_query_2_samples,
        },
        "large_query": {
            "samples": large_query_samples,
        },
        "large_query_2": {
            "samples": large_query_2_samples,
        }
    });

    write_to_file(FILE_PATH, summary.to_string().as_bytes());
}

fn start_server() -> Connection {
    // Delete container from before if present.
    match Command::new("sh")
        .arg("-c")
        .arg(format!("docker rm {}", CONTAINER_NAME))
        .output()
        .expect("unable to delete container")
        .status
        .success()
    {
        true => {}
        false => println!("unable to delete container"),
    }

    // Start the new server instance.
    match Command::new("sh")
        .arg("-c")
        .arg(format!(
            "docker run --rm -d -p 7687:7687 --name {} {} --telemetry-enabled=False",
            CONTAINER_NAME, MEMGRAPH_VERSION
        ))
        .output()
        .expect("failed to start server")
        .status
        .success()
    {
        true => {}
        false => panic!("failed to start server"),
    }

    // Wait until server has started.
    loop {
        match Connection::connect(&ConnectParams {
            host: Some(String::from("localhost")),
            autocommit: true,
            ..Default::default()
        }) {
            Ok(connection) => {
                return connection;
            }
            Err(_) => {
                thread::sleep(time::Duration::from_millis(100));
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
    setup: &dyn Fn(&mut Connection) -> Result<(), MgError>,
) -> Vec<f64> {
    let mut connection = start_server();
    if let Err(err) = setup(&mut connection) {
        panic!("{}", err)
    }

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
        println!("Another benchmark rep DONE");
    }

    stop_server();

    samples
}

fn write_to_file(file_name: &str, data: &[u8]) {
    let path = Path::new(file_name);
    if let Some(p) = path.parent() {
        create_dir_all(p).expect("Unable to create dirs")
    };
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .expect("unable to write to file");
    file.write_all(data).expect("unable to write to file");
}

fn insert_query_benchmark() -> Vec<f64> {
    let times = benchmark_query("CREATE (u:User)", None, &|_| Ok(()));
    println!("insert_query_benchmark DONE");
    times
}

fn create_index(connection: &mut Connection, index: &str) -> Result<(), MgError> {
    connection.execute(format!("CREATE INDEX ON {}", index).as_str(), None)?;
    connection.fetchall()?;
    Ok(())
}

fn small_query_with_query_params_benchmark() -> Vec<f64> {
    let times = benchmark_query(
        "MATCH (u:User) WHERE u.name = $name RETURN u",
        Some(&hashmap! {
            String::from("name") => QueryParam::String(String::from("u")),
        }),
        &|connection| {
            connection.execute("CREATE (u:User {name: 'u'})", None)?;
            connection.fetchall()?;

            create_index(connection, ":User(name)")
        },
    );
    println!("small_query_with_query_params_benchmark DONE");
    times
}

fn small_query_with_query_params_2_benchmark() -> Vec<f64> {
    let times = benchmark_query(
        "MATCH (u:User) WHERE u.id = $id RETURN u",
        Some(&hashmap! {
            String::from("id") => QueryParam::Int(1),
        }),
        &|connection| {
            let mut str = String::new();
            for _ in 0..100 {
                str.push('a');
            }
            connection.execute(
                format!("CREATE (u:User {{id: 1, name: '{}'}})", str).as_str(),
                None,
            )?;
            connection.fetchall()?;

            create_index(connection, ":User(id)")
        },
    );
    println!("small_query_with_query_params_2_benchmark DONE");
    times
}

fn large_query_benchmark() -> Vec<f64> {
    let times = benchmark_query("MATCH (u:User) RETURN u", None, &|connection| {
        for i in 0..1000 {
            connection.execute(format!("CREATE (u:User {{id: {}}})", i,).as_str(), None)?;
            connection.fetchall()?;
        }
        Ok(())
    });
    println!("large_query_benchmark DONE");
    times
}

fn large_query_2_benchmark() -> Vec<f64> {
    let times = benchmark_query("MATCH (u:User) RETURN u", None, &|connection| {
        let mut name = String::new();
        for _ in 0..100 {
            name.push('a');
        }
        for i in 0..100 {
            connection.execute(
                format!("CREATE (u:User {{id: {}, name: '{}'}})", i, name).as_str(),
                None,
            )?;
            connection.fetchall()?;
        }
        Ok(())
    });
    println!("large_query_2_benchmark DONE");
    times
}
