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

use rsmgclient::{connect, ConnectParams, MgValue, QueryParam};
use std::collections::HashMap;

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

fn main() {
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
        trust_callback: Some(&my_callback),
        ..Default::default()
    };

    let connection = match connect(&connect_prms) {
        Ok(c) => c,
        Err(err) => panic!("{}", err),
    };
    let mut map: HashMap<String, QueryParam> = HashMap::new();
    map.insert(String::from("address"), QueryParam::Null);
    map.insert(String::from("is_programmer"), QueryParam::Bool(true));
    map.insert(
        String::from("name"),
        QueryParam::String(String::from("James Bond")),
    );
    map.insert(
        String::from("list"),
        QueryParam::List(vec![QueryParam::String(String::from("val"))]),
    );

    let mut params: HashMap<String, QueryParam> = HashMap::new();
    params.insert(String::from("real_params"), QueryParam::Map(map));

    let rows: Vec<Vec<MgValue>> = match connection.execute(
        "CREATE (n:Person {name: 'John'})-[e:KNOWS]->(m:Person {name: 'Steve'}) RETURN n, e, m;",
        Some(&params),
    ) {
        Ok(res) => res,
        Err(err) => panic!("Query failed: {}", err),
    };

    for row in rows {
        for val in row {
            println!("{}", val);
        }
    }
}
