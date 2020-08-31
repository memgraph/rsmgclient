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

use rsmgclient::{ConnectParams, Connection, MgError};

fn execute_query() -> Result<(), MgError> {
    let connect_params = ConnectParams {
        host: Some(String::from("localhost")),
        ..Default::default()
    };
    let mut connection = Connection::connect(&connect_params)?;

    let query =
        "CREATE (u:User {name: 'Alice'})-[l:Likes]->(m:Software {name: 'Memgraph'}) RETURN u, l, m";
    let columns = connection.execute(query, None)?;
    println!("Columns: {}", columns.join(", "));

    let records = connection.fetchall()?;
    for value in &records[0].values {
        println!("{}", value);
    }

    connection.commit()?;
    Ok(())
}

fn main() {
    if let Err(error) = execute_query() {
        panic!("{}", error)
    }
}
