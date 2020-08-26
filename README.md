# rsmgclient - Rust Memgraph Client

[![](https://github.com/memgraph/rsmgclient/workflows/CI/badge.svg)](https://github.com/memgraph/rsmgclient/actions)

`rsmgclient` is a Memgraph database adapter for Rust programming language.

rsmgclient module is the current implementation of the adapter. It is implemented in C as a wrapper 
around [mgclient](https://github.com/memgraph/mgclient), the official Memgraph client library.

## Prerequisites

### Installation

`rsmgclient` is a C wrapper around the [mgclient](https://github.com/memgraph/mgclient) Memgraph 
client library. To install it from sources you will need:
   - [Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html) - 1.42.0 or above
   - A C compiler supporting C11 standard
   - [mgclient](https://github.com/memgraph/mgclient) has to be installed because `rsmgclient` statically links `mgclient`
   - [Memgraph](https://docs.memgraph.com/memgraph/quick-start)

Once prerequisites are met, if you want to use it as library for your own Rust project, you can 
install rsmgclient using `cargo` to download it from crates.io:
```
$ cargo install rsmgclient
```

### Building from source

To use `rsmgclient` for contributing or just looking closely how it is made, you will need:
   - Cloned [rsmgclient](https://github.com/memgraph/rsmgclient) repository
   - [Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html) - 1.42.0-nightly or above
   - A C compiler supporting C11 standard
   - [mgclient](https://github.com/memgraph/mgclient)
   - [Memgraph](https://docs.memgraph.com/memgraph/quick-start)

Once rsmgclient is installed, you will need to build it and then you can run the test suite to verify 
it is working correctly.

```
$ cargo build
$ cargo test
```

## Documentation

Online documentation can be found on [docs.rs pages](https://docs.rs/rsmgclient/).

## Code sample

Here is an example of an interactive session showing some of the basic commands:

```rust
use rsmgclient::{ConnectParams, Connection};


fn main(){
    // Parameters for connecting to database.
    let connect_params = ConnectParams {
        host: Some(String::from("localhost")),
        ..Default::default()
    };

    // Make a connection to the database.
    let mut connection = match Connection::connect(&connect_params) {
        Ok(c) => c,
        Err(err) => panic!("{}", err)
    };

    // Execute a query.
    let query = "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'}) RETURN u, m";
    match connection.execute(query, None) {
        Ok(columns) => println!("Columns: {}", columns.join(", ")),
        Err(err) => panic!("{}", err)
    };

    // Fetch all query results.
    match connection.fetchall() {
        Ok(records) => {
            for value in &records[0].values {
                println!("{}", value);
            }
        },
        Err(err) => panic!("{}", err)
    };


    // Commit any pending transaction to the database.
    match connection.commit() {
        Ok(()) => {},
        Err(err) => panic!("{}", err)
    };
}
```
