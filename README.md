# rsmgclient - Rust Memgraph Client

[![](https://github.com/memgraph/rsmgclient/workflows/CI/badge.svg)](https://github.com/memgraph/rsmgclient/actions)

`rsmgclient` is [Memgraph](https://memgraph.com/) database adapter for Rust
programming language. `rsmgclient` crate is the current implementation of the
adapter. It is implemented as a wrapper around
[mgclient](https://github.com/memgraph/mgclient), the official Memgraph C/C++
client library.

## Installation

### Prerequisites

- [Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)
  1.42.0 or above
- Prerequisites of [mgclient](https://github.com/memgraph/mgclient):
  - A C compiler supporting C11 standard
  - CMake 3.8 or newer
  - OpenSSL 1.0.2 or newer

### Installing from crates.io

Once prerequisites are met, if you want to use `rsmgclient` as library for your
own Rust project, you can install it by using `cargo`:

```bash
cargo install rsmgclient
```

### Building from Source

To contribute into `rsmgclient` or just looking closely how it is made,
you will need:

- Cloned [rsmgclient](https://github.com/memgraph/rsmgclient) repository
- [Memgraph Quick Start Guide](https://memgraph.com/docs/memgraph/quick-start)

Once `rsmgclient` is cloned, you will need to build it and then you can run
the test suite to verify it is working correctly:

```bash
git submodule update --init
cargo build
# Run Memgraph based on the quick start guide
cargo test
```

## Documentation

Online documentation can be found on [docs.rs
pages](https://docs.rs/rsmgclient/).

## Code Sample

`src/main.rs` is an example showing some of the basic commands:

```rust
use rsmgclient::{ConnectParams, Connection, MgError, Value};

fn execute_query() -> Result<(), MgError> {
    // Connect to Memgraph.
    let connect_params = ConnectParams {
        host: Some(String::from("localhost")),
        ..Default::default()
    };
    let mut connection = Connection::connect(&connect_params)?;

    // Create simple graph.
    connection.execute_without_results(
        "CREATE (p1:Person {name: 'Alice'})-[l1:Likes]->(m:Software {name: 'Memgraph'}) \
         CREATE (p2:Person {name: 'John'})-[l2:Likes]->(m);",
    )?;

    // Fetch the graph.
    let columns = connection.execute("MATCH (n)-[r]->(m) RETURN n, r, m;", None)?;
    println!("Columns: {}", columns.join(", "));
    for record in connection.fetchall()? {
        for value in record.values {
            match value {
                Value::Node(node) => print!("{}", node),
                Value::Relationship(edge) => print!("-{}-", edge),
                value => print!("{}", value),
            }
        }
        println!();
    }
    connection.commit()?;

    Ok(())
}

fn main() {
    if let Err(error) = execute_query() {
        panic!("{}", error)
    }
}
```
