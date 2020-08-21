# rsmgclient - Rust Memgraph Client

[![](https://github.com/memgraph/rsmgclient/workflows/CI/badge.svg)](https://github.com/memgraph/rsmgclient/actions)

`rsmgclient` is a Rust binding for [mgclient](https://github.com/memgraph/mgclient) used to interact with [Memgraph](https://memgraph.com/).

`rsmgclient` is a Memgraph database adapter for Rust programming language.

mgclient module is the current implementation of the adapter. It is implementedin C as a wrapper around [mgclient](https://github.com/memgraph/mgclient),
the official Memgraph client library.

## Prerequisites

### Build prerequisites

`rsmgclient` is a C wrapper around the [mgclient](https://github.com/memgraph/mgclient) Memgraph client library. To install it from sources you will need:
   - Rust
   - A C compiler supporting C11 standard
   - mgclient header files
   - [Memgraph](https://docs.memgraph.com/memgraph/quick-start)

Once prerequisites are met, you can install rsmgclient using `cargo` to download it from crates.io:
```
$ cargo install rsmgclient
```

Once mgclient is installed, you can run the test suite to verify it
is working correctly.

```
$ cargo test
```

## Documentation

Online documentation can be found on [GitHub
pages](https://memgraph.github.io/rsmgclient/).

## Code sample

Here is an example of an interactive session showing some of the basic commands:

```rust
use rsmgclient::{ConnectParams, Connection};

// Parameters for connecting to database
let connect_params = ConnectParams {
    host: Some(String::from("localhost")),
    ..Default::default()
};

// Make a connection to the database
let mut connection = match Connection::connect(&connect_params) {
    Ok(c) => c,
    Err(err) => panic!("{}", err)
};

// Execute a query
let query = "CREATE (u:User {name: 'Alice'})-[:Likes]->(m:Software {name: 'Memgraph'}) RETURN u, m";
match connection.execute(query, None) {
    Ok(columns) => println!("Columns: {}", columns.join(", ")),
    Err(err) => panic!("{}", err)
};

// Fetch all query results
match connection.fetchall() {
    Ok(records) => {
        for value in records[0].values {
            println!("{}", value);
        }
    },
    Err(err) => panic!("{}", err)
};


// Commit any pending transaction to the database
match connection.commit() {
    Ok(()) => {},
    Err(err) => panic!("{}", err)
};
```