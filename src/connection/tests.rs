use super::*;
use crate::bindings;
use crate::value::Node;
use serial_test::serial;

struct Mockery {
    mg_session_params_make_ctx: bindings::mock_params_make::__mg_session_params_make::Context,
    mg_connect_ctx: bindings::mock_connect::__mg_connect::Context,
    mg_session_run_ctx: bindings::mock_run::__mg_session_run::Context,
    mg_session_pull_ctx: bindings::mock_pull::__mg_session_pull::Context,
    mg_session_error_ctx: bindings::mock_mg_session_error::__mg_session_error::Context,
    mg_result_row_ctx: bindings::mock_mg_result_row::__mg_result_row::Context,
}

impl Default for Mockery {
    fn default() -> Self {
        Mockery {
            mg_session_params_make_ctx: {
                let ctx = bindings::mock_params_make::mg_session_params_make_context();
                ctx.expect()
                    .returning(|| unsafe { bindings::mg_session_params_make() });
                ctx
            },
            mg_connect_ctx: {
                let ctx = bindings::mock_connect::mg_connect_context();
                ctx.expect()
                    .returning(|arg1, arg2| unsafe { bindings::mg_connect(arg1, arg2) });
                ctx
            },
            mg_session_run_ctx: {
                let ctx = bindings::mock_run::mg_session_run_context();
                ctx.expect().returning(|arg1, arg2, arg3, arg4| unsafe {
                    bindings::mg_session_run(arg1, arg2, arg3, arg4)
                });
                ctx
            },
            mg_session_pull_ctx: {
                let ctx = bindings::mock_pull::mg_session_pull_context();
                ctx.expect()
                    .returning(|arg1, arg2| unsafe { bindings::mg_session_pull(arg1, arg2) });
                ctx
            },
            mg_session_error_ctx: {
                let ctx = bindings::mock_mg_session_error::mg_session_error_context();
                ctx.expect()
                    .returning(|arg1| unsafe { bindings::mg_session_error(arg1) });
                ctx
            },
            mg_result_row_ctx: {
                let ctx = bindings::mock_mg_result_row::mg_result_row_context();
                ctx.expect()
                    .returning(|arg1| unsafe { bindings::mg_result_row(arg1) });
                ctx
            },
        }
    }
}

fn run_test(test: fn(mock: &mut Mockery)) {
    let mut mock = Mockery {
        ..Default::default()
    };
    initialize();
    test(&mut mock);
}

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

fn execute_query(query: String) {
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
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            trust_callback: Some(&my_callback),
            lazy: false,
            sslcert: Some(String::from("test_sslcert")),
            ..Default::default()
        };
        let _connection = get_connection(connect_prms);
    });
}

#[test]
#[serial]
fn error_while_pulling_fetchall() {
    run_test(|mut mockery| {
        let mut connection = get_connection(ConnectParams {
            address: Some(String::from("127.0.0.1")),
            ..Default::default()
        });

        let query =
            "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'}) RETURN u, m";
        let result = connection.execute(query, None);
        assert!(result.is_ok());

        mockery.mg_session_pull_ctx = bindings::mock_pull::mg_session_pull_context();
        mockery
            .mg_session_pull_ctx
            .expect()
            .returning(|_arg1, _arg2| 4);

        mockery.mg_result_row_ctx = bindings::mock_mg_result_row::mg_result_row_context();
        mockery
            .mg_result_row_ctx
            .expect()
            .returning(|_arg1| std::ptr::null_mut());

        mockery.mg_session_error_ctx = bindings::mock_mg_session_error::mg_session_error_context();
        mockery
            .mg_session_error_ctx
            .expect()
            .returning(|_arg1| str_to_c_str("error"));

        let result = connection.fetchall();
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), String::from("error"));
        assert_eq!(connection.status, ConnectionStatus::Bad);
    });
}

#[test]
#[serial]
fn error_while_pulling_fetchmany() {
    run_test(|mut mockery| {
        let mut connection = get_connection(ConnectParams {
            address: Some(String::from("127.0.0.1")),
            ..Default::default()
        });

        let query =
            "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'}) RETURN u, m";
        let result = connection.execute(query, None);
        assert!(result.is_ok());

        mockery.mg_session_pull_ctx = bindings::mock_pull::mg_session_pull_context();
        mockery
            .mg_session_pull_ctx
            .expect()
            .returning(|_arg1, _arg2| 4);

        mockery.mg_result_row_ctx = bindings::mock_mg_result_row::mg_result_row_context();
        mockery
            .mg_result_row_ctx
            .expect()
            .returning(|_arg1| std::ptr::null_mut());

        mockery.mg_session_error_ctx = bindings::mock_mg_session_error::mg_session_error_context();
        mockery
            .mg_session_error_ctx
            .expect()
            .returning(|_arg1| str_to_c_str("error"));

        let result = connection.fetchmany(Some(3));
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), String::from("error"));
        assert_eq!(connection.status, ConnectionStatus::Bad);
    });
}

#[test]
#[serial]
fn error_while_connecting() {
    run_test(|mut mockery| {
        let mut connection = get_connection(ConnectParams {
            address: Some(String::from("127.0.0.1")),
            ..Default::default()
        });

        let query =
            "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'}) RETURN u, m";
        let result = connection.execute(query, None);
        assert!(result.is_ok());

        mockery.mg_session_run_ctx = bindings::mock_run::mg_session_run_context();
        mockery
            .mg_session_run_ctx
            .expect()
            .returning(|_arg1, _arg2, _arg3, _arg4| 4);

        mockery.mg_session_pull_ctx = bindings::mock_pull::mg_session_pull_context();
        mockery
            .mg_session_pull_ctx
            .expect()
            .returning(|_arg1, _arg2| 4);

        mockery.mg_result_row_ctx = bindings::mock_mg_result_row::mg_result_row_context();
        mockery
            .mg_result_row_ctx
            .expect()
            .returning(|_arg1| std::ptr::null_mut());

        mockery.mg_session_error_ctx = bindings::mock_mg_session_error::mg_session_error_context();
        mockery
            .mg_session_error_ctx
            .expect()
            .returning(|_arg1| str_to_c_str("error"));

        let result = connection.fetchone();
        connection.status=ConnectionStatus::Ready;
        let commit = connection.commit();
        connection.status=ConnectionStatus::Ready;
        let rollback = connection.rollback();
        assert!(rollback.is_err());
        assert!(commit.is_err());
        assert!(result.is_err());
        assert_eq!(commit.err().unwrap().to_string(), String::from("error"));
        assert_eq!(rollback.err().unwrap().to_string(), String::from("error"));
        assert_eq!(connection.status, ConnectionStatus::Bad);
    });
}

#[test]
#[serial]
#[should_panic(expected = "both sslcert and sslkey should be provided")]
fn from_connect_fetchone_panic_sslkey() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            trust_callback: Some(&my_callback),
            lazy: false,
            sslkey: Some(String::from("test_sslkey")),
            ..Default::default()
        };
        let _connection = get_connection(connect_prms);
    });
}

#[test]
#[serial]
fn from_connect_fetchone() {
    run_test(|_mock| {
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
    });
}

#[test]
#[serial]
#[should_panic(expected = "Query failed: Parameter $name not provided.")]
fn from_connect_fetchone_none_params() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
fn from_connect_fetchone_address() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            ..Default::default()
        };
        let connection = get_connection(connect_prms);
        assert_eq!(connection.lazy, true);
    });
}

#[test]
#[serial]
#[should_panic(expected = "explicit panic")]
fn from_connect_fetchone_explicit_panic() {
    run_test(|_mock| {
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
    });
}

#[test]
#[serial]
#[should_panic(expected = "Connection is closed")]
fn from_connect_fetchone_closed_panic() {
    run_test(|_mock| {
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
        let mut connection = get_connection(connect_prms);
        let params = get_params("name".to_string(), "Alice".to_string());

        let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
        match connection.execute(&query, Some(&params)) {
            Ok(x) => x,
            Err(err) => panic!("Query failed: {}", err),
        };
        connection.status=ConnectionStatus::Closed;
        loop {
            match connection.fetchone() {
                Ok(_res) => {}
                Err(err) => panic!("Fetch failed: {}", err),
            }
        }
    });
}

#[test]
#[serial]
#[should_panic(expected = "Bad connection")]
fn from_connect_fetchone_bad_panic() {
    run_test(|_mock| {
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
    });
}

#[test]
#[serial]
fn from_connect_fetchmany() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
fn from_connect_fetchmany_error() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
fn from_connect_fetchall() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
fn from_connect_panic_fetchall() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
#[should_panic(expected = "Fetching failed: Connection is not executing")]
fn from_connect_fetchall_panic() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
#[should_panic(expected = "Connection is already executing")]
fn from_connect_fetchall_executing_panic() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
#[should_panic(expected = "Bad connection")]
fn from_connect_fetchall_bad_panic() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
#[should_panic(expected = "Connection is closed")]
fn from_connect_fetchall_closed_panic() {
    run_test(|_mock| {
        execute_query(String::from(
            "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})",
        ));
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            lazy: true,
            ..Default::default()
        };
        let params = get_params("name".to_string(), "Alice".to_string());
        let mut connection = get_connection(connect_prms);
        connection.close();
        let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
        match connection.execute(&query, Some(&params)) {
            Ok(_x) => {}
            Err(err) => panic!("{}", err),
        }
    });
}

#[test]
#[serial]
fn from_connect_fetchone_summary() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
fn from_connect_fetchone_summary_none() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            lazy: true,
            ..Default::default()
        };
        let connection = get_connection(connect_prms);
        let summary = connection.summary();
        assert_eq!(true, summary.is_none());
    });
}

#[test]
#[serial]
fn from_connect_fetchall_commit() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
#[should_panic(expected = "Connection is closed")]
fn from_connect_fetchall_commit_panic_closed() {
    run_test(|_mock| {
        execute_query(String::from(
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

        connection.close();
        match connection.commit() {
            Ok(_x) => {}
            Err(err) => panic!("Fetching failed: {}", err),
        }
    });
}

#[test]
#[serial]
#[should_panic(expected = "Can't commit while executing")]
fn from_connect_fetchall_commit_panic_executing() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
#[should_panic(expected = "Not in transaction")]
fn from_connect_fetchall_commit_panic_transaction() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
fn from_connect_fetchall_rollback() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
#[should_panic(expected = "Connection is closed")]
fn from_connect_fetchall_rollback_panic_closed() {
    run_test(|_mock| {
        execute_query(String::from(
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

        connection.close();
        match connection.rollback() {
            Ok(_x) => {}
            Err(err) => panic!("Fetching failed: {}", err),
        }
    });
}

#[test]
#[serial]
#[should_panic(expected = "Can't commit while executing")]
fn from_connect_fetchall_rollback_panic_executing() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
#[should_panic(expected = "Not in transaction")]
fn from_connect_fetchall_rollback_panic_transaction() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
fn from_connect_fetchall_set_get_lazy() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
#[should_panic(expected = "Can't set lazy while executing")]
fn from_connect_fetchall_set_get_lazy_panic_executing() {
    run_test(|_mock| {
        execute_query(String::from(
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
    });
}

#[test]
#[serial]
#[should_panic(expected = "Bad connection")]
fn from_connect_fetchall_set_get_lazy_panic_bad() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            lazy: true,
            ..Default::default()
        };
        let mut connection = get_connection(connect_prms);

        connection.status = ConnectionStatus::Bad;
        connection.set_lazy(false);
        assert_eq!(false, connection.lazy);
    });
}

#[test]
#[serial]
#[should_panic(expected = "Connection is closed")]
fn from_connect_fetchall_set_get_lazy_panic_closed() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            lazy: true,
            ..Default::default()
        };
        let mut connection = get_connection(connect_prms);

        connection.close();
        connection.set_lazy(false);
        assert_eq!(false, connection.lazy);
    });
}

#[test]
#[serial]
fn from_connect_fetchall_set_get_autocommit() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            ..Default::default()
        };
        let mut connection = get_connection(connect_prms);

        connection.set_autocommit(true);
        assert_eq!(true, connection.autocommit());
    });
}

#[test]
#[serial]
#[should_panic(expected = "Can't set autocommit while in pending transaction")]
fn from_connect_fetchall_set_get_autocommit_panic_transaction() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            ..Default::default()
        };
        let mut connection = get_connection(connect_prms);

        connection.in_transaction = true;
        connection.set_autocommit(true);
        assert_eq!(true, connection.autocommit());
    });
}

#[test]
#[serial]
#[should_panic(expected = "Can't set autocommit while executing")]
fn from_connect_fetchall_set_get_autocommit_panic_executing() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            ..Default::default()
        };
        let mut connection = get_connection(connect_prms);

        connection.status = ConnectionStatus::Executing;
        connection.set_autocommit(true);
        assert_eq!(true, connection.autocommit());
    });
}

#[test]
#[serial]
#[should_panic(expected = "Bad connection")]
fn from_connect_fetchall_set_get_autocommit_panic_bad() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            ..Default::default()
        };
        let mut connection = get_connection(connect_prms);

        connection.status = ConnectionStatus::Bad;
        connection.set_autocommit(true);
        assert_eq!(true, connection.autocommit());
    });
}

#[test]
#[serial]
#[should_panic(expected = "Connection is closed")]
fn from_connect_fetchall_set_get_autocommit_panic_closed() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            ..Default::default()
        };
        let mut connection = get_connection(connect_prms);

        connection.close();
        connection.set_autocommit(true);
        assert_eq!(true, connection.autocommit());
    });
}

#[test]
#[serial]
fn from_connect_fetchall_set_get_arraysize() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            ..Default::default()
        };
        let mut connection = get_connection(connect_prms);

        connection.set_arraysize(2);
        assert_eq!(2, connection.arraysize());
    });
}

#[test]
#[serial]
fn from_connect_fetchall_get_lazy_transaction_status() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            ..Default::default()
        };
        let connection = get_connection(connect_prms);

        assert_eq!(true, connection.lazy());
        assert_eq!(false, connection.in_transaction());
        assert_eq!(&ConnectionStatus::Ready, connection.status());
    });
}

#[test]
#[serial]
fn from_connect_close() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            ..Default::default()
        };
        let mut connection = get_connection(connect_prms);

        connection.close();
        assert_eq!(&ConnectionStatus::Closed, connection.status());
    });
}

#[test]
#[serial]
#[should_panic(expected = "Connection is executing")]
fn from_connect_close_panic() {
    run_test(|_mock| {
        let connect_prms = ConnectParams {
            address: Some(String::from("127.0.0.1")),
            ..Default::default()
        };
        let mut connection = get_connection(connect_prms);

        connection.status = ConnectionStatus::Executing;
        connection.close();
        assert_eq!(&ConnectionStatus::Closed, connection.status());
    });
}
