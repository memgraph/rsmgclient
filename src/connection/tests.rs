use super::*;
use crate::{Node, Value};
use serial_test::serial;

fn get_connection(prms: &ConnectParams) -> Connection {
    match Connection::connect(prms) {
        Ok(c) => c,
        Err(err) => panic!("Creating connection failed: {}", err),
    }
}

fn execute_query(connection: &mut Connection, query: &str) -> Vec<String> {
    match connection.execute(query, None) {
        Ok(x) => x,
        Err(err) => panic!("Executing query failed: {}", err),
    }
}

fn execute_query_and_fetchall(query: &str) -> Vec<Record> {
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        autocommit: true,
        ..Default::default()
    };
    let mut connection = get_connection(&connect_prms);
    assert_eq!(connection.status, ConnectionStatus::Ready);

    match connection.execute(query, None) {
        Ok(x) => x,
        Err(err) => panic!("Executing query failed: {}", err),
    };
    assert_eq!(connection.status, ConnectionStatus::Executing);

    match connection.fetchall() {
        Ok(records) => {
            assert_eq!(connection.status, ConnectionStatus::Ready);
            records
        }
        Err(err) => panic!("Fetching all failed: {}", err),
    }
}

fn initialize() -> Connection {
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        ..Default::default()
    };
    let mut connection = get_connection(&connect_prms);
    assert_eq!(connection.status, ConnectionStatus::Ready);

    let query = String::from("MATCH (n) DETACH DELETE n;");
    match connection.execute(&query, None) {
        Ok(x) => x,
        Err(err) => panic!("Executing delete all query failed: {}", err),
    };
    assert_eq!(connection.status, ConnectionStatus::Executing);

    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetching all failed: {}", err),
    }
    assert_eq!(connection.status, ConnectionStatus::InTransaction);

    match connection.commit() {
        Ok(_) => {}
        Err(err) => panic!("Commit failed: {}", err),
    }
    assert_eq!(connection.status, ConnectionStatus::Ready);

    get_connection(&connect_prms)
}

fn create_node(labels: Vec<String>, properties: HashMap<String, Value>) -> Node {
    Node {
        id: 0,
        label_count: labels.len() as u32,
        labels,
        properties,
    }
}

fn assert_eq_nodes(n1: &Node, n2: &Node) {
    assert_eq!(n1.label_count, n2.label_count);
    assert_eq!(n1.labels, n2.labels);
    assert_eq!(n1.properties, n2.properties);
}

fn create_params(key: String, value: String) -> HashMap<String, QueryParam> {
    let mut params: HashMap<String, QueryParam> = HashMap::new();
    params.insert(key, QueryParam::String(value));
    params
}

#[allow(clippy::ptr_arg)]
fn my_callback(host: &String, ip_address: &String, key_type: &String, fingerprint: &String) -> i32 {
    assert_eq!(host, "localhost");
    assert_eq!(ip_address, "127.0.0.1");
    assert_eq!(key_type, "rsaEncryption");
    assert_eq!(fingerprint.len(), 128);

    0
}

#[test]
#[serial]
#[should_panic(expected = "both sslcert and sslkey should be provided")]
fn panic_sslcert() {
    initialize();
    let connect_prms = ConnectParams {
        address: Some(String::from("127.0.0.1")),
        trust_callback: Some(&my_callback),
        lazy: false,
        sslcert: Some(String::from("test_sslcert")),
        ..Default::default()
    };
    get_connection(&connect_prms);
}

#[test]
#[serial]
#[should_panic(expected = "both sslcert and sslkey should be provided")]
fn panic_sslkey() {
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

fn test_execute_error(connection: &mut Connection, error: &str) {
    let result = connection.execute("RETURN 1;", None);
    assert!(result.is_err());
    assert!(format!("{}", result.err().unwrap()).contains(error));
}

#[test]
#[serial]
fn execute_executing_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Executing;
    test_execute_error(&mut connection, "executing");
}

#[test]
#[serial]
fn execute_fetching_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Fetching;
    test_execute_error(&mut connection, "fetching");
}

#[test]
#[serial]
fn execute_closed_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Closed;
    test_execute_error(&mut connection, "is closed");
}

#[test]
#[serial]
fn execute_bad_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Bad;
    test_execute_error(&mut connection, "is bad");
}

#[test]
#[serial]
fn parameter_provided() {
    let mut connection = initialize();

    match connection.execute(
        "RETURN $name;",
        Some(&create_params("name".to_string(), "test".to_string())),
    ) {
        Ok(columns) => {
            assert_eq!(columns.len(), 1);
        }
        Err(err) => panic!("Query failed: {}", err),
    };
    let records = connection.fetchall().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].values.len(), 1);
    let value = &records[0].values[0];
    assert_eq!(
        match value {
            Value::String(s) => s,
            _ => panic!("Given parameter is not a string"),
        },
        "test"
    );
}

#[test]
#[serial]
#[should_panic(expected = "Query failed: Parameter $name not provided.")]
fn parameter_not_provided() {
    let mut connection = initialize();

    match connection.execute("MATCH (n) WHERE n.name = $name RETURN n;", None) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
}

fn test_fetchone_person_alice(connection: &mut Connection) {
    execute_query_and_fetchall("CREATE (n:Person {name: 'Alice'});");
    let columns = execute_query(connection, "MATCH (n) RETURN n;");
    assert_eq!(columns.join(", "), "n");
    let record = connection.fetchone().unwrap().unwrap();
    assert_eq!(record.values.len(), 1);
    match &record.values[0] {
        Value::Node(n) => {
            assert_eq_nodes(
                n,
                &create_node(
                    vec!["Person".to_string()],
                    hashmap! {"name".to_string() => Value::String("Alice".to_string())},
                ),
            );
        }
        _ => panic!("Fetch one didn't return the expected node"),
    };
    assert!(connection.fetchone().unwrap().is_none());
    assert!(connection.fetchone().is_err());
}

#[test]
#[serial]
fn fetchone_lazy() {
    let mut connection = initialize();

    test_fetchone_person_alice(&mut connection);
}

#[test]
#[serial]
fn fetchone_not_lazy() {
    let mut connection = initialize();

    connection.set_lazy(false);
    assert!(!connection.lazy);

    test_fetchone_person_alice(&mut connection);
}

#[test]
#[serial]
fn fetchone_no_data() {
    let mut connection = initialize();

    execute_query(&mut connection, "MATCH (n:NoData) RETURN n;");
    let first = connection.fetchone();
    if let Ok(rec) = first {
        assert!(rec.is_none());
    } else {
        panic!("First fetched record should be None")
    }
}

#[test]
#[serial]
fn fetchone_summary() {
    let mut connection = initialize();

    execute_query_and_fetchall("CREATE (), ();");

    execute_query(&mut connection, "MATCH (n) RETURN n;");
    loop {
        match connection.fetchone() {
            Ok(res) => match res {
                Some(x) => for _val in &x.values {},
                None => break,
            },
            Err(err) => panic!("Fetch one unexpectedly failed: {}", err),
        }
    }

    let summary = connection.summary().unwrap();
    assert_eq!(7, summary.len());
    for key in &[
        "cost_estimate",
        "has_more",
        "parsing_time",
        "type",
        "planning_time",
        "plan_execution_time",
        "run_id",
    ] {
        assert!(summary.contains_key(key as &str));
    }
}

#[test]
#[serial]
fn fetchone_summary_none() {
    let connection = initialize();
    let summary = connection.summary();
    assert!(summary.is_none());
}

fn test_fetchone_error(connection: &mut Connection, error: &str) {
    let result = connection.fetchone();
    assert!(result.is_err());
    assert!(format!("{}", result.err().unwrap()).contains(error));
}

#[test]
#[serial]
fn fetchone_ready_error() {
    let mut connection = initialize();
    test_fetchone_error(&mut connection, "ready");
}

#[test]
#[serial]
fn fetchone_in_transaction_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::InTransaction;
    test_fetchone_error(&mut connection, "in transaction");
}

#[test]
#[serial]
fn fetchone_closed_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Closed;
    test_fetchone_error(&mut connection, "is closed");
}

#[test]
#[serial]
fn fetchone_bad_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Bad;
    test_fetchone_error(&mut connection, "is bad");
}

fn test_fetchmany_empty_nodes(connection: &mut Connection) {
    execute_query_and_fetchall("CREATE (), (), ();");

    let columns = execute_query(connection, "MATCH (n) RETURN n;");
    assert_eq!(columns.join(", "), "n");

    match connection.fetchmany(Some(2)) {
        Ok(records) => {
            assert_eq!(records.len(), 2);
        }
        Err(err) => panic!("Fetch many unexpectedly failed: {}", err),
    }
    assert_eq!(connection.status, ConnectionStatus::Fetching);

    match connection.fetchmany(Some(2)) {
        Ok(records) => {
            assert_eq!(records.len(), 1);
        }
        Err(err) => panic!("Fetch many unexpectedly failed: {}", err),
    }
    assert_eq!(connection.status, ConnectionStatus::InTransaction);
}

#[test]
#[serial]
fn fetchmany_lazy() {
    let mut connection = initialize();

    test_fetchmany_empty_nodes(&mut connection);
}

#[test]
#[serial]
fn fetchmany_not_lazy() {
    let mut connection = initialize();

    connection.set_lazy(false);
    assert!(!connection.lazy);

    test_fetchmany_empty_nodes(&mut connection);
}

fn test_fetchall_empty_nodes(connection: &mut Connection) {
    execute_query_and_fetchall("CREATE (), (), ();");

    let columns = execute_query(connection, "MATCH (n) RETURN n;");
    assert_eq!(columns.join(", "), "n");

    match connection.fetchall() {
        Ok(records) => {
            assert_eq!(records.len(), 3);
        }
        Err(err) => panic!("Fetch all unexpectedly failed: {}", err),
    }
    assert_eq!(connection.status, ConnectionStatus::InTransaction);
}

#[test]
#[serial]
fn fetchall_lazy() {
    let mut connection = initialize();

    test_fetchall_empty_nodes(&mut connection);
}

#[test]
#[serial]
fn fetchall_not_lazy() {
    let mut connection = initialize();

    connection.set_lazy(false);
    assert!(!connection.lazy);

    test_fetchmany_empty_nodes(&mut connection);
}

fn test_commit_error(connection: &mut Connection, error: &str) {
    let commit_res = connection.commit();
    assert!(commit_res.is_err());
    assert!(format!("{}", commit_res.err().unwrap()).contains(error));
}

#[test]
#[serial]
fn commit_executing_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Executing;
    test_commit_error(&mut connection, "executing");
}

#[test]
#[serial]
fn commit_fetching_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Fetching;
    test_commit_error(&mut connection, "fetching");
}

#[test]
#[serial]
fn commit_closed_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Closed;
    test_commit_error(&mut connection, "is closed");
}

#[test]
#[serial]
fn commit_bad_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Bad;
    test_commit_error(&mut connection, "is bad");
}

#[test]
#[serial]
fn rollback() {
    let mut connection = initialize();

    execute_query(&mut connection, "MATCH (n) RETURN n;");

    match connection.fetchall() {
        Ok(_records) => {}
        Err(err) => panic!("Fetch all unexpectedly failed: {}", err),
    }
    assert_eq!(connection.status, ConnectionStatus::InTransaction);

    match connection.rollback() {
        Ok(_x) => {}
        Err(err) => panic!("Rollback unexpectedly failed: {}", err),
    }
    assert_eq!(connection.status, ConnectionStatus::Ready);
}

fn test_rollback_error(connection: &mut Connection, error: &str) {
    let rollback_res = connection.rollback();
    assert!(rollback_res.is_err());
    assert!(format!("{}", rollback_res.err().unwrap()).contains(error));
}

#[test]
#[serial]
fn rollback_ready_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Ready;
    test_rollback_error(&mut connection, "in transaction");
}

#[test]
#[serial]
fn rollback_executing_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Executing;
    test_rollback_error(&mut connection, "executing");
}

#[test]
#[serial]
fn rollback_fetching_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Fetching;
    test_rollback_error(&mut connection, "fetching");
}

#[test]
#[serial]
fn rollback_closed_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Closed;
    test_rollback_error(&mut connection, "is closed");
}

#[test]
#[serial]
fn rollback_bad_error() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Bad;
    test_rollback_error(&mut connection, "is bad");
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
#[should_panic(expected = "Can't set lazy while connection is closed")]
fn set_lazy_closed() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Closed;
    connection.set_lazy(false);
}

#[test]
#[serial]
#[should_panic(expected = "Can't set lazy while connection is bad")]
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
#[should_panic(expected = "Can't set autocommit while connection is closed")]
fn set_autocommit_closed() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Closed;
    connection.set_autocommit(true);
}

#[test]
#[serial]
#[should_panic(expected = "Can't set autocommit while connection is bad")]
fn set_autocommit_bad() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Bad;
    connection.set_autocommit(true);
}

#[test]
#[serial]
fn fetchall_set_get_arraysize() {
    let mut connection = initialize();
    connection.set_arraysize(2);
    assert_eq!(2, connection.arraysize());
}

#[test]
#[serial]
fn close() {
    let mut connection = initialize();
    connection.close();
    assert_eq!(ConnectionStatus::Closed, connection.status());
}

#[test]
#[serial]
#[should_panic(expected = "Can't close while executing")]
fn executing_close_panic() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Executing;
    connection.close();
}

#[test]
#[serial]
#[should_panic(expected = "Can't close while fetching")]
fn fetching_close_panic() {
    let mut connection = initialize();
    connection.status = ConnectionStatus::Fetching;
    connection.close();
}

#[test]
#[serial]
fn execute_without_results() {
    let mut connection = initialize();

    assert!(connection
        .execute_without_results("CREATE (n1) CREATE (n2) RETURN n1, n2;")
        .is_ok());
    assert_eq!(ConnectionStatus::Ready, connection.status());

    assert!(connection.execute("MATCH (n) RETURN n;", None).is_ok());
    assert_eq!(ConnectionStatus::Executing, connection.status());
    match connection.fetchall() {
        Ok(records) => assert_eq!(records.len(), 2),
        Err(err) => panic!("Failed to get data after execute without results {}.", err),
    }
    assert_eq!(ConnectionStatus::InTransaction, connection.status());
}
