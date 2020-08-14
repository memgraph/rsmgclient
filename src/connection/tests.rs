use super::*;
use crate::{Node, Value};
use serial_test::serial;

pub fn initialize() {
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    let query = String::from("MATCH (n) DETACH DELETE n");
    match connection.execute(&query, None) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
    match connection.fetchall() {
        Ok(records) => {
            for record in records {
                for _val in &record.values {}
            }
        }
        Err(err) => panic!("Fetching failed: {}", err),
    }

    let query = String::from("CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})");
    match connection.execute(&query, None) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
    match connection.fetchall() {
        Ok(records) => {
            for record in records {
                for _val in &record.values {}
            }
        }
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
    println!("host: {}", host);
    println!("ip_address: {}", ip_address);
    println!("key_type: {}", key_type);
    println!("fingerprint: {}", fingerprint);

    0
}

#[test]
#[serial]
#[should_panic(expected = "both sslcert and sslkey should be provided")]
fn from_connect_fetchone_panic_sslcert() {
    initialize();
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
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
        host: Some(String::from("localhost")),
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
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
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
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
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
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
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
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
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
fn from_connect_fetchmany() {
    initialize();
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
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
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
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
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
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
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
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
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
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
#[should_panic(expected = "Connection is closed")]
fn from_connect_fetchall_closed_panic() {
    initialize();
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
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
