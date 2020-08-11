// Copyright (c) 2016-2020 Memgraph Ltd. [https://memgraph.com]
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use rsmgclient::{ConnectParams, Connection, QueryParam};
use std::collections::HashMap;

fn main() {
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
        lazy: false,
        ..Default::default()
    };

    let mut connection = match Connection::connect(&connect_prms) {
        Ok(c) => c,
        Err(err) => panic!("{}", err),
    };

    let mut params: HashMap<String, QueryParam> = HashMap::new();
    params.insert(
        String::from("name"),
        QueryParam::String(String::from("Alice")),
    );

    let query = String::from("MATCH (n:User) WHERE n.name = $name RETURN n LIMIT 5");
    // let query = String::from("CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'})");
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
