use super::*;
use crate::{Node, Value};
use serial_test::serial;

pub fn initialize() {
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    let query = String::from("MATCH (n) DETACH DELETE n");
    match connection.execute(&query, None) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

fn fill_database(query: String) {
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);
    match connection.execute(&query, None) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

fn get_connection(prms: ConnectParams) -> Connection {
    match Connection::connect(&prms) {
        Ok(c) => c,
        Err(err) => panic!("{}", err),
    }
}

fn get_params(str_value: String, qrp: String) -> HashMap<String, QueryParam> {
    let mut params: HashMap<String, QueryParam> = HashMap::new();
    params.insert(str_value, QueryParam::String(qrp));
    params
}

pub fn my_callback(
    host: &String,
    ip_address: &String,
    key_type: &String,
    fingerprint: &String,
) -> i32 {
    assert_eq!(host, "localhost");
    assert_eq!(ip_address, "127.0.0.1");
    assert_eq!(key_type, "rsaEncryption");
    assert_eq!(fingerprint.len(), 128);

    0
}

#[test]
#[serial]
#[should_panic(expected = "both sslcert and sslkey should be provided")]
fn from_connect_fetchone_panic_sslcert() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        trust_callback: Some(&my_callback),
        lazy: false,
        sslcert: Some(String::from("test_sslcert")),
        ..Default::default()
    };
    let _connection = get_connection(connect_prms);
}

#[test]
#[serial]
#[should_panic(expected = "both sslcert and sslkey should be provided")]
fn from_connect_fetchone_panic_sslkey() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        trust_callback: Some(&my_callback),
        lazy: false,
        sslkey: Some(String::from("test_sslkey")),
        ..Default::default()
    };
    let _connection = get_connection(connect_prms);
}

#[test]
#[serial]
fn from_connect_fetchone() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        trust_callback: Some(&my_callback),
        lazy: false,
        username: Some(String::from("test_username")),
        password: Some(String::from("test_password")),
        client_name: String::from("test_username test_password"),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);
    let params = get_params("name".to_string(), "Alice".to_string());

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    let columns = match connection.execute(&query, Some(&params)) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
    assert_eq!(columns.join(", "), "n");
    assert_eq!(connection.lazy, false);

    loop {
        match connection.fetchone() {
            Ok(res) => match res {
                Some(x) => {
                    for val in &x.values {
                        let values = vec![String::from("User")];
                        let mg_map = hashmap! {
                            String::from("name") => Value::String("Alice".to_string()),
                        };
                        let node = Value::Node(Node {
                            id: match val {
                                Value::Node(x) => x.id,
                                _ => 1,
                            },
                            label_count: 1,
                            labels: values,
                            properties: mg_map,
                        });
                        assert_eq!(&node, val);
                    }
                }
                None => break,
            },
            Err(err) => panic!("Fetch failed: {}", err),
        }
    }
}

#[test]
#[serial]
#[should_panic(expected = "Query failed: Parameter $name not provided.")]
fn from_connect_fetchone_none_params() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, None) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
}

#[test]
#[serial]
fn from_connect_fetchone_address() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let connection = get_connection(connect_prms);
    assert_eq!(connection.lazy, true);
}

#[test]
#[serial]
#[should_panic(expected = "explicit panic")]
fn from_connect_fetchone_explicit_panic() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        trust_callback: Some(&my_callback),
        lazy: false,
        username: Some(String::from("test_username")),
        password: Some(String::from("test_password")),
        client_name: String::from("test_username test_password"),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);
    let params = get_params("name".to_string(), "Alice".to_string());

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
    connection.results_iter = None;
    loop {
        match connection.fetchone() {
            Ok(res) => match res {
                Some(x) => {
                    for val in &x.values {
                        let values = vec![String::from("User")];
                        let mg_map = hashmap! {
                            String::from("name") => Value::String("Alice".to_string()),
                        };
                        let node = Value::Node(Node {
                            id: match val {
                                Value::Node(x) => x.id,
                                _ => 1,
                            },
                            label_count: 1,
                            labels: values,
                            properties: mg_map,
                        });
                        assert_eq!(&node, val);
                    }
                }
                None => break,
            },
            Err(err) => panic!("Fetch failed: {}", err),
        }
    }
}

#[test]
#[serial]
#[should_panic(expected = "Connection is closed")]
fn from_connect_fetchone_closed_panic() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        trust_callback: Some(&my_callback),
        lazy: false,
        username: Some(String::from("test_username")),
        password: Some(String::from("test_password")),
        client_name: String::from("test_username test_password"),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);
    let params = get_params("name".to_string(), "Alice".to_string());

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
    connection.status = ConnectionStatus::Closed;
    loop {
        match connection.fetchone() {
            Ok(_res) => {}
            Err(err) => panic!("Fetch failed: {}", err),
        }
    }
}

#[test]
#[serial]
#[should_panic(expected = "Bad connection")]
fn from_connect_fetchone_bad_panic() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        trust_callback: Some(&my_callback),
        lazy: false,
        username: Some(String::from("test_username")),
        password: Some(String::from("test_password")),
        client_name: String::from("test_username test_password"),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);
    let params = get_params("name".to_string(), "Alice".to_string());

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
    connection.status = ConnectionStatus::Bad;
    loop {
        match connection.fetchone() {
            Ok(_res) => {}
            Err(err) => panic!("Fetch failed: {}", err),
        }
    }
}

#[test]
#[serial]
fn from_connect_fetchmany() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: false,
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);
    let params = get_params("name".to_string(), "Alice".to_string());

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("Query failed: {}", err),
    };

    loop {
        let size = 3;
        match connection.fetchmany(Some(size)) {
            Ok(res) => {
                for record in &res {
                    for val in &record.values {
                        let values = vec![String::from("User")];
                        let mg_map = hashmap! {
                            String::from("name") => Value::String("Alice".to_string()),
                        };
                        let node = Value::Node(Node {
                            id: match val {
                                Value::Node(x) => x.id,
                                _ => 1,
                            },
                            label_count: 1,
                            labels: values,
                            properties: mg_map,
                        });
                        assert_eq!(&node, val);
                    }
                }
                if res.len() != size as usize {
                    break;
                }
            }
            Err(err) => panic!("Fetch failed: {}", err),
        }
    }
}

#[test]
#[serial]
fn from_connect_fetchmany_error() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: false,
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);
    let params = get_params("name".to_string(), "Alice".to_string());

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("Query failed: {}", err),
    };

    loop {
        let size = 3;
        match connection.fetchmany(None) {
            Ok(res) => {
                for record in &res {
                    for val in &record.values {
                        let values = vec![String::from("User")];
                        let mg_map = hashmap! {
                            String::from("name") => Value::String("Alice".to_string()),
                        };
                        let node = Value::Node(Node {
                            id: match val {
                                Value::Node(x) => x.id,
                                _ => 1,
                            },
                            label_count: 1,
                            labels: values,
                            properties: mg_map,
                        });
                        assert_eq!(&node, val);
                    }
                }
                if res.len() != size as usize {
                    break;
                }
            }
            Err(err) => panic!("Fetch failed: {}", err),
        }
    }
}

#[test]
#[serial]
fn from_connect_fetchall() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);
    let params = get_params("name".to_string(), "Alice".to_string());

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    match connection.fetchall() {
        Ok(records) => {
            for record in records {
                for val in &record.values {
                    let values = vec![String::from("User")];
                    let mg_map = hashmap! {
                        String::from("name") => Value::String("Alice".to_string()),
                    };
                    let node = Value::Node(Node {
                        id: match val {
                            Value::Node(x) => x.id,
                            _ => 1,
                        },
                        label_count: 1,
                        labels: values,
                        properties: mg_map,
                    });
                    assert_eq!(&node, val);
                }
            }
        }
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
fn from_connect_panic_fetchall() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);
    let params = get_params("name".to_string(), "Alice".to_string());

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");

    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    match connection.fetchall() {
        Ok(records) => {
            for record in records {
                for val in &record.values {
                    let values = vec![String::from("User")];
                    let mg_map = hashmap! {
                        String::from("name") => Value::String("Alice".to_string()),
                    };
                    let node = Value::Node(Node {
                        id: match val {
                            Value::Node(x) => x.id,
                            _ => 1,
                        },
                        label_count: 1,
                        labels: values,
                        properties: mg_map,
                    });
                    assert_eq!(&node, val);
                }
            }
        }
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Fetching failed: Connection is not executing")]
fn from_connect_fetchall_panic() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);
    match connection.fetchall() {
        Ok(records) => {
            for record in records {
                for val in &record.values {
                    let values = vec![String::from("User")];
                    let mg_map = hashmap! {
                        String::from("name") => Value::String("Alice".to_string()),
                    };
                    let node = Value::Node(Node {
                        id: match val {
                            Value::Node(x) => x.id,
                            _ => 1,
                        },
                        label_count: 1,
                        labels: values,
                        properties: mg_map,
                    });
                    assert_eq!(&node, val);
                }
            }
        }
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Connection is already executing")]
fn from_connect_fetchall_executing_panic() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let params = get_params("name".to_string(), "Alice".to_string());
    let mut connection = get_connection(connect_prms);
    connection.status = ConnectionStatus::Executing;
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Bad connection")]
fn from_connect_fetchall_bad_panic() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let params = get_params("name".to_string(), "Alice".to_string());
    let mut connection = get_connection(connect_prms);
    connection.status = ConnectionStatus::Bad;
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Connection is closed")]
fn from_connect_fetchall_closed_panic() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let params = get_params("name".to_string(), "Alice".to_string());
    let mut connection = get_connection(connect_prms);
    connection.status = ConnectionStatus::Closed;
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }
}

#[test]
#[serial]
fn from_connect_fetchone_summary() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let params = get_params("name".to_string(), "Alice".to_string());
    let mut connection = get_connection(connect_prms);
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    loop {
        match connection.fetchone() {
            Ok(res) => match res {
                Some(x) => for _val in &x.values {},
                None => break,
            },
            Err(err) => panic!("Fetch failed: {}", err),
        }
    }

    let summary = connection.summary().unwrap();
    assert_eq!(5, summary.len());
}

#[test]
#[serial]
fn from_connect_fetchone_summary_none() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let connection = get_connection(connect_prms);
    let summary = connection.summary();
    assert_eq!(true,summary.is_none());
}

#[test]
#[serial]
fn from_connect_fetchall_commit() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let params = get_params("name".to_string(), "Alice".to_string());
    let mut connection = get_connection(connect_prms);
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }

    connection.status = ConnectionStatus::Ready;
    match connection.commit() {
        Ok(_x) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Connection is closed")]
fn from_connect_fetchall_commit_panic_closed() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let params = get_params("name".to_string(), "Alice".to_string());
    let mut connection = get_connection(connect_prms);
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }

    connection.status = ConnectionStatus::Closed;
    match connection.commit() {
        Ok(_x) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Can't commit while executing")]
fn from_connect_fetchall_commit_panic_executing() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let params = get_params("name".to_string(), "Alice".to_string());
    let mut connection = get_connection(connect_prms);
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }

    connection.status = ConnectionStatus::Executing;
    match connection.commit() {
        Ok(_x) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Not in transaction")]
fn from_connect_fetchall_commit_panic_transaction() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let params = get_params("name".to_string(), "Alice".to_string());
    let mut connection = get_connection(connect_prms);
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }

    connection.in_transaction = false;
    match connection.commit() {
        Ok(_x) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
fn from_connect_fetchall_rollback() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let params = get_params("name".to_string(), "Alice".to_string());
    let mut connection = get_connection(connect_prms);
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }

    connection.in_transaction = true;
    match connection.rollback() {
        Ok(_x) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Connection is closed")]
fn from_connect_fetchall_rollback_panic_closed() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let params = get_params("name".to_string(), "Alice".to_string());
    let mut connection = get_connection(connect_prms);
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }

    connection.status = ConnectionStatus::Closed;
    match connection.rollback() {
        Ok(_x) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Can't commit while executing")]
fn from_connect_fetchall_rollback_panic_executing() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let params = get_params("name".to_string(), "Alice".to_string());
    let mut connection = get_connection(connect_prms);
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }

    connection.status = ConnectionStatus::Executing;
    match connection.rollback() {
        Ok(_x) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Bad connection")]
fn from_connect_fetchall_rollback_panic_bad() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let params = get_params("name".to_string(), "Alice".to_string());
    let mut connection = get_connection(connect_prms);
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }

    connection.status = ConnectionStatus::Bad;
    match connection.rollback() {
        Ok(_x) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Not in transaction")]
fn from_connect_fetchall_rollback_panic_transaction() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let params = get_params("name".to_string(), "Alice".to_string());
    let mut connection = get_connection(connect_prms);
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }

    connection.in_transaction = false;
    match connection.rollback() {
        Ok(_x) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
fn from_connect_fetchall_set_get_lazy() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    connection.set_lazy(false);
    assert_eq!(false, connection.lazy);
}

#[test]
#[serial]
#[should_panic(expected = "Can't set lazy while executing")]
fn from_connect_fetchall_set_get_lazy_panic_executing() {
    initialize();
    fill_database(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    connection.status = ConnectionStatus::Executing;
    connection.set_lazy(false);
    assert_eq!(false, connection.lazy);
}

#[test]
#[serial]
#[should_panic(expected = "Bad connection")]
fn from_connect_fetchall_set_get_lazy_panic_bad() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    connection.status = ConnectionStatus::Bad;
    connection.set_lazy(false);
    assert_eq!(false, connection.lazy);
}

#[test]
#[serial]
#[should_panic(expected = "Connection is closed")]
fn from_connect_fetchall_set_get_lazy_panic_closed() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        lazy: true,
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    connection.status = ConnectionStatus::Closed;
    connection.set_lazy(false);
    assert_eq!(false, connection.lazy);
}

#[test]
#[serial]
fn from_connect_fetchall_set_get_autocommit() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    connection.set_autocommit(true);
    assert_eq!(true, connection.autocommit());
}

#[test]
#[serial]
#[should_panic(expected = "Can't set autocommit while in pending transaction")]
fn from_connect_fetchall_set_get_autocommit_panic_transaction() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    connection.in_transaction = true;
    connection.set_autocommit(true);
    assert_eq!(true, connection.autocommit());
}

#[test]
#[serial]
#[should_panic(expected = "Can't set autocommit while executing")]
fn from_connect_fetchall_set_get_autocommit_panic_executing() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    connection.status = ConnectionStatus::Executing;
    connection.set_autocommit(true);
    assert_eq!(true, connection.autocommit());
}

#[test]
#[serial]
#[should_panic(expected = "Bad connection")]
fn from_connect_fetchall_set_get_autocommit_panic_bad() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    connection.status = ConnectionStatus::Bad;
    connection.set_autocommit(true);
    assert_eq!(true, connection.autocommit());
}

#[test]
#[serial]
#[should_panic(expected = "Connection is closed")]
fn from_connect_fetchall_set_get_autocommit_panic_closed() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    connection.status = ConnectionStatus::Closed;
    connection.set_autocommit(true);
    assert_eq!(true, connection.autocommit());
}

#[test]
#[serial]
fn from_connect_fetchall_set_get_arraysize() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    connection.set_arraysize(2);
    assert_eq!(2, connection.arraysize());
}

#[test]
#[serial]
fn from_connect_fetchall_get_lazy_transaction_status() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let connection = get_connection(connect_prms);

    assert_eq!(true, connection.lazy());
    assert_eq!(false, connection.in_transaction());
    assert_eq!(&ConnectionStatus::Ready, connection.status());
}

#[test]
#[serial]
fn from_connect_close() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    connection.close();
    assert_eq!(&ConnectionStatus::Closed, connection.status());
}

#[test]
#[serial]
#[should_panic(expected = "Connection is executing")]
fn from_connect_close_panic() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    connection.status = ConnectionStatus::Executing;
    connection.close();
    assert_eq!(&ConnectionStatus::Closed, connection.status());
}
