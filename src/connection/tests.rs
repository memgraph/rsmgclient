use super::*;

fn get_connection(prms: ConnectParams) -> Connection {
    let connection = match Connection::connect(&prms) {
        Ok(c) => c,
        Err(err) => panic!("{}", err),
    };
    connection
}

fn get_params() -> HashMap<String, QueryParam> {
    let mut params: HashMap<String, QueryParam> = HashMap::new();
    params.insert(
        String::from("name"),
        QueryParam::String(String::from("Alice")),
    );
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
#[should_panic(expected = "both sslcert and sslkey should be provided")]
fn from_connect_fetchone_panic() {
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
fn from_connect_fetchone() {
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
    let params = get_params();

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    //let query = String::from("CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})");
    let columns = match connection.execute(&query, Some(&params)) {
        Ok(x) => x,
        Err(err) => panic!("Query failed: {}", err),
    };
    println!("Columns: {}", columns.join(", "));

    loop {
        match connection.fetchone() {
            Ok(res) => match res {
                Some(x) => {
                    println!("Number of rows: 1");
                    print!("Row: ");
                    for val in &x.values {
                        print!("val: {}    ", val);
                    }
                    println!();
                }
                None => break,
            },
            Err(err) => panic!("Fetch failed: {}", err),
        }
    }
}

#[test]
fn from_connect_fetchmany() {
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
        lazy: false,
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);
    let params = get_params();

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("Query failed: {}", err),
    };

    loop {
        let size = 3;
        match connection.fetchmany(Some(size)) {
            Ok(res) => {
                println!("Number of rows: {}", res.len());
                for record in &res {
                    print!("Row: ");
                    for val in &record.values {
                        print!("val: {}  ", val);
                    }
                    println!();
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
fn from_connect_fetchmany_error() {
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
        lazy: false,
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);
    let params = get_params();

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("Query failed: {}", err),
    };

    loop {
        let size = 3;
        match connection.fetchmany(None) {
            Ok(res) => {
                println!("Number of rows: {}", res.len());
                for record in &res {
                    print!("Row: ");
                    for val in &record.values {
                        print!("val: {}  ", val);
                    }
                    println!();
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
fn from_connect_fetchall() {
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
        lazy: true,
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);
    let params = get_params();

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    match connection.execute(&query, Some(&params)) {
        Ok(_x) => {}
        Err(err) => panic!("{}", err),
    }

    match connection.fetchall() {
        Ok(records) => {
            println!("Number of rows: {}", records.len());
            for record in records {
                print!("Row: ");
                for val in &record.values {
                    print!("val: {}    ", val);
                }
                println!();
            }
        }
        Err(err) => panic!("Fetching failed: {}", err),
    }
}

#[test]
#[should_panic(expected = "Fetching failed: Connection is not executing")]
fn from_connect_fetchall_panic() {
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
        lazy: true,
        ..Default::default()
    };
    let mut connection = get_connection(connect_prms);

    match connection.fetchall() {
        Ok(records) => {
            println!("Number of rows: {}", records.len());
            for record in records {
                print!("Row: ");
                for val in &record.values {
                    print!("val: {}    ", val);
                }
                println!();
            }
        }
        Err(err) => panic!("Fetching failed: {}", err),
    }
}
