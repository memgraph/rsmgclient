use criterion::{Criterion, black_box, criterion_group, criterion_main, BatchSize};
use rsmgclient::{Connection, ConnectParams, QueryParam};
use maplit::{hashmap};

fn create_default_connection() -> Connection {
    let connect_params = ConnectParams {
        host: Some(String::from("localhost")),
        lazy: false,
        ..Default::default()
    };
    let connection = match Connection::connect(&connect_params) {
        Ok(c) => c,
        Err(err) => panic!("Failed to connect: {}", err)
    };
    connection
}

fn insert_benchmark(c: &mut Criterion) {
    let insert_query = "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})";
    let mut connection = create_default_connection();

    c.bench_function("insert benchmark", |b| b.iter(|| {
        match connection.execute(insert_query, None) {
            Ok(cols) => cols,
            Err(_err) => panic!()
        };
        match connection.fetchall() {
            Ok(vals) => vals,
            Err(_err) => panic!()
        };
    }));

    connection.execute("MATCH (n) DETACH DELETE n", None);
    connection.fetchall();
}

fn query_benchmark_small(c: &mut Criterion) {
    let mut connection = create_default_connection();

    connection.execute("CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})", None);
    connection.fetchall();

    let small_query = "MATCH (u:User) WHERE u.name = $name RETURN u";
    let query_params = hashmap! {
        String::from("name") => QueryParam::String(String::from("Alice")),
    };
    c.bench_function("small query benchmark", |b| b.iter(
        || {
            match connection.execute(small_query, Some(&query_params)) {
                Ok(cols) => cols,
                Err(_err) => panic!()
            };
            match connection.fetchall() {
                Ok(vals) => vals,
                Err(_err) => panic!()
            };
        }));

    connection.execute("MATCH (n) DETACH DELETE n", None);
    connection.fetchall();
}

fn query_benchmark_large(c: &mut Criterion) {
    let mut connection = create_default_connection();

    for i in 0..100{
        connection.execute(format!("CREATE (u:User {{name: 'Alice{}'}})-[:Likes]->(m:Software {{name: 'Memgraph{}'}})",i,i).as_str(), None);
        connection.fetchall();
    }
    

    let large_query = "MATCH (u:User) RETURN u";
    c.bench_function("large query benchmark", |b| b.iter(
        || {
            match connection.execute(large_query, None) {
                Ok(cols) => cols,
                Err(_err) => panic!()
            };
            match connection.fetchall() {
                Ok(vals) => vals,
                Err(_err) => panic!()
            };
        }));

    connection.execute("MATCH (n) DETACH DELETE n", None);
    connection.fetchall();
}

criterion_group!(benches, insert_benchmark, query_benchmark_small, query_benchmark_large);
criterion_main!(benches);