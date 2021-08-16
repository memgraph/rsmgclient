use super::*;
use crate::{Node, Value};
use serial_test::serial;

fn get_connection(prms: &ConnectParams) -> Connection {
    match Connection::connect(&prms) {
        Ok(c) => c,
        Err(err) => panic!("{}", err),
    }
}

pub fn initialize() -> Connection {
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };

    let mut connection = get_connection(&connect_prms);
    let query = String::from("MATCH (n) DETACH DELETE n");
    match connection.execute(&query, None) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
    match connection.commit() {
        Ok(_) => {}
        Err(err) => panic!("Commit failed: {}", err),
    }

    get_connection(&connect_prms)
}

fn execute_query(query: String) {
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };

    let mut connection = get_connection(&connect_prms);
    match connection.execute(&query, None) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

fn get_params(str_value: String, qrp: String) -> HashMap<String, QueryParam> {
    let mut params: HashMap<String, QueryParam> = HashMap::new();
    params.insert(str_value, QueryParam::String(qrp));
    params
}

#[allow(clippy::ptr_arg)]
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
    let _connection = get_connection(&connect_prms);
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
    let _connection = get_connection(&connect_prms);
}

#[test]
#[serial]
fn from_connect_fetchone() {
    let mut connection = initialize();
    connection.set_lazy(false);

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let params = get_params("name".to_string(), "Alice".to_string());

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    let columns = match connection.execute(&query, Some(&params)) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
    assert_eq!(columns.join(", "), "n");
    assert!(!connection.lazy);

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
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, None) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
}

#[test]
#[serial]
fn from_connect_fetchone_no_data() {
    initialize();
    execute_query(String::from(
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
    let mut connection = get_connection(&connect_prms);
    let params = get_params("name".to_string(), "Something".to_string());

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };

    let first = connection.fetchone();
    if let Ok(rec) = first {
        assert!(rec.is_none());
    } else {
        panic!("First fetched record should be None")
    }
}

#[test]
#[serial]
#[should_panic(expected = "Fetch failed: Can't call fetchone if connection is closed")]
fn from_connect_fetchone_closed_panic() {
    let mut connection = initialize();

    connection.status = ConnectionStatus::Closed;
    match connection.fetchone() {
        Ok(_res) => {}
        Err(err) => panic!("Fetch failed: {}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Fetch failed: Can't call fetchone if connection is bad")]
fn from_connect_fetchone_bad_panic() {
    initialize();
    execute_query(String::from(
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
    let mut connection = get_connection(&connect_prms);
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
    let mut connection = initialize();
    connection.set_lazy(false);

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
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
    let mut connection = initialize();
    connection.set_lazy(false);

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
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
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
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
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));

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
#[should_panic(expected = "Fetching failed: Can't call fetchone while ready")]
fn from_connect_fetchall_panic() {
    let mut connection = initialize();

    match connection.fetchall() {
        Ok(_) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Can't call execute while already executing")]
fn from_connect_fetchall_executing_panic() {
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let params = get_params("name".to_string(), "Alice".to_string());
    connection.status = ConnectionStatus::Executing;
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Can't call execute while connection is bad")]
fn from_connect_fetchall_bad_panic() {
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let params = get_params("name".to_string(), "Alice".to_string());
    connection.status = ConnectionStatus::Bad;
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }
}

#[test]
#[serial]
#[should_panic(expected = "Can't call execute while connection is closed")]
fn from_connect_fetchall_closed_panic() {
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let params = get_params("name".to_string(), "Alice".to_string());
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
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let params = get_params("name".to_string(), "Alice".to_string());
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
    assert_eq!(6, summary.len());
    for key in &[
        "cost_estimate",
        "has_more",
        "parsing_time",
        "type",
        "planning_time",
        "plan_execution_time",
    ] {
        assert!(summary.contains_key(&key as &str));
    }
}

#[test]
#[serial]
fn from_connect_fetchone_summary_none() {
    let connection = initialize();
    let summary = connection.summary();
    assert!(summary.is_none());
}

#[test]
#[serial]
fn from_connect_fetchall_commit() {
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let params = get_params("name".to_string(), "Alice".to_string());
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
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let params = get_params("name".to_string(), "Alice".to_string());
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
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let params = get_params("name".to_string(), "Alice".to_string());
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
fn from_connect_fetchall_rollback() {
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let params = get_params("name".to_string(), "Alice".to_string());
    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }

    connection.status = ConnectionStatus::InTransaction;
    match connection.rollback() {
        Ok(_x) => {}
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[serial]
fn from_connect_fetchall_rollback_panic_closed() {
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let params = get_params("name".to_string(), "Alice".to_string());
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
    let rollback_res = connection.rollback();
    assert!(rollback_res.is_err());
    assert!(format!("{}", rollback_res.err().unwrap()).contains("is closed"));
}

#[test]
#[serial]
#[should_panic(expected = "Fetching failed: Can't rollback while executing")]
fn from_connect_fetchall_rollback_panic_executing() {
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let params = get_params("name".to_string(), "Alice".to_string());
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
    let mut connection = initialize();

    execute_query(String::from(
        "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
    ));
    let params = get_params("name".to_string(), "Alice".to_string());
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
fn set_lazy() {
    let mut connection = initialize();
    connection.set_lazy(false);
    assert!(!connection.lazy);
}

#[test]
#[serial]
#[should_panic(expected = "Can't set lazy while in transaction")]
fn set_lazy_in_transaction() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::InTransaction;
    connection.set_lazy(false);
}

#[test]
#[serial]
#[should_panic(expected = "Can't set lazy while executing")]
fn set_lazy_executing() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Executing;
    connection.set_lazy(false);
}

#[test]
#[serial]
#[should_panic(expected = "Can't set lazy while fetching")]
fn set_lazy_fetching() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Fetching;
    connection.set_lazy(false);
}

#[test]
#[serial]
#[should_panic(expected = "Can't set lazy because connection is closed")]
fn set_lazy_closed() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Closed;
    connection.set_lazy(false);
}

#[test]
#[serial]
#[should_panic(expected = "Can't set lazy because connection is bad")]
fn set_lazy_bad() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Bad;
    connection.set_lazy(false);
}

#[test]
#[serial]
fn set_autocommit() {
    let mut connection = initialize();
    connection.set_autocommit(true);
    assert!(connection.autocommit());
}

#[test]
#[serial]
#[should_panic(expected = "Can't set autocommit while in transaction")]
fn set_autocommit_in_transaction() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::InTransaction;
    connection.set_autocommit(true);
}

#[test]
#[serial]
#[should_panic(expected = "Can't set autocommit while executing")]
fn set_autocommit_executing() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Executing;
    connection.set_autocommit(true);
}

#[test]
#[serial]
#[should_panic(expected = "Can't set autocommit while fetching")]
fn set_autocommit_fetching() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Fetching;
    connection.set_autocommit(true);
}

#[test]
#[serial]
#[should_panic(expected = "Can't set autocommit because connection is closed")]
fn set_autocommit_closed() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Closed;
    connection.set_autocommit(true);
}

#[test]
#[serial]
#[should_panic(expected = "Can't set autocommit because connection is bad")]
fn set_autocommit_bad() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Bad;
    connection.set_autocommit(true);
}

#[test]
#[serial]
fn from_connect_fetchall_set_get_arraysize() {
    let mut connection = initialize();
    connection.set_arraysize(2);
    assert_eq!(2, connection.arraysize());
}

#[test]
#[serial]
fn from_connect_fetchall_get_lazy_transaction_status() {
    // TODO(gitbuda): Fix/remove this test becuase it's just checking defaults.
    let connection = initialize();
    assert!(connection.lazy());
    assert!(connection.status != ConnectionStatus::InTransaction);
    assert_eq!(&ConnectionStatus::Ready, connection.status());
}

#[test]
#[serial]
fn from_connect_close() {
    let mut connection = initialize();
    connection.close();
    assert_eq!(ConnectionStatus::Closed, *connection.status());
}

#[test]
#[serial]
#[should_panic(expected = "Can't close while executing")]
fn from_connect_executing_close_panic() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Executing;
    connection.close();
}

#[test]
#[serial]
#[should_panic(expected = "Can't close while fetching")]
fn from_connect_fetching_close_panic() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Fetching;
    connection.close();
}

#[test]
#[serial]
fn from_connect_execute_without_results() {
    let mut connection = initialize();

    assert!(connection
        .execute_without_results("CREATE (n1) CREATE (n2) RETURN n1, n2;")
        .is_ok());
    assert_eq!(&ConnectionStatus::Ready, connection.status());

    assert!(connection.execute("MATCH (n) RETURN n;", None).is_ok());
    assert_eq!(&ConnectionStatus::Executing, connection.status());
    match connection.fetchall() {
        Ok(records) => assert_eq!(records.len(), 2),
        Err(err) => panic!("Failed to get data after execute without results {}.", err),
    }
    assert_eq!(&ConnectionStatus::InTransaction, connection.status());
}
