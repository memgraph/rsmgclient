use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use maplit::hashmap;
use rsmgclient::{ConnectParams, Connection, QueryParam};
use std::time::{Duration, Instant};
use std::{thread, time};
use std::process::Command;

fn create_default_connection() -> Connection {
    let connect_params = ConnectParams {
        host: Some(String::from("localhost")),
        lazy: false,
        ..Default::default()
    };
    let connection = match Connection::connect(&connect_params) {
        Ok(c) => c,
        Err(err) => panic!("Failed to connect: {}", err),
    };
    connection
}

fn clear_and_restart_memgraph(mut connection: Connection){
    connection.execute("MATCH (n) DETACH DELETE n", None);
    connection.fetchall();

    //Change the parameter after echo into your password
    let output = Command::new("sh")
            .arg("-c")
            .arg("echo 0909 | sudo -S systemctl restart memgraph")
            .output()
            .expect("failed to execute process");

    let wait = time::Duration::from_millis(1000);
    let now = time::Instant::now();
    thread::sleep(wait);
}

fn insert_benchmark(c: &mut Criterion) {
    let insert_query = "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})";
    let mut connection = create_default_connection();
    c.bench_function("insert benchmark", |b| {
        b.iter(|| {
            connection.execute(insert_query,None);
            match connection.fetchall() {
                Ok(vals) => vals,
                Err(_err) => panic!(),
            };
        })
    });

    clear_and_restart_memgraph(connection);
}

fn query_benchmark_small(c: &mut Criterion) {
    let mut connection = create_default_connection();

    connection.execute(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
        None,
    );
    connection.fetchall();

    let small_query = "MATCH (u:User) WHERE u.name = $name RETURN u";
    let query_params = hashmap! {
        String::from("name") => QueryParam::String(String::from("Alice")),
    };
    c.bench_function("small query benchmark", |b| {
        b.iter(|| {
            match connection.execute(small_query, Some(&query_params)) {
                Ok(cols) => cols,
                Err(_err) => panic!(),
            };
            match connection.fetchall() {
                Ok(vals) => vals,
                Err(_err) => panic!(),
            };
        })
    });

    connection.execute("MATCH (n) DETACH DELETE n", None);
    connection.fetchall();
}

fn query_benchmark_large(c: &mut Criterion) {
    let mut connection = create_default_connection();
    let large_query = "MATCH (u:User) RETURN u";

    for i in 0..100 {
        connection.execute(
            format!(
                "CREATE (u:User {{name: 'Alice{}'}})-[:Likes]->(m:Software {{name: 'Memgraph{}'}})",
                i, i
            )
            .as_str(),
            None,
        );
        connection.fetchall();
    }

    c.bench_function("large query benchmark", |b| {
        b.iter(|| {
            match connection.execute(large_query, None) {
                Ok(cols) => cols,
                Err(_err) => panic!(),
            };
            match connection.fetchall() {
                Ok(vals) => vals,
                Err(_err) => panic!(),
            };
        })
    });

    clear_and_restart_memgraph(connection);
}

criterion_group!(
    benches,
    insert_benchmark,
    query_benchmark_small,
    query_benchmark_large
);
criterion_main!(benches);
