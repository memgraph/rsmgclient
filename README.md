# rsmgclient - Rust Memgraph Client

[![](https://github.com/memgraph/rsmgclient/workflows/CI/badge.svg)](https://github.com/memgraph/rsmgclient/actions)

`rsmgclient` is a Memgraph database adapter for Rust programming language.

rsmgclient crate is the current implementation of the adapter. It is
implemented as a wrapper around
[mgclient](https://github.com/memgraph/mgclient), the official Memgraph client
library.

## Prerequisites

### Installation

`rsmgclient` is a wrapper around the
[mgclient](https://github.com/memgraph/mgclient) Memgraph client library. To
install it from sources you will need:

- [Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)
  1.42.0 or above
- A C compiler supporting C11 standard
- [mgclient](https://github.com/memgraph/mgclient) has to be installed
  because `rsmgclient` statically links `mgclient`
- [Memgraph](https://docs.memgraph.com/memgraph/quick-start)

Once prerequisites are met, if you want to use it as library for your own Rust
project, you can install rsmgclient using `cargo` to download it from
crates.io:

```bash
cargo install rsmgclient
```

### Building from source

To use `rsmgclient` for contributing or just looking closely how it is made,
you will need:

- Cloned [rsmgclient](https://github.com/memgraph/rsmgclient) repository
- [Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)
  1.42.0-nightly or above
- A C compiler supporting C11 standard
- [mgclient](https://github.com/memgraph/mgclient)
- [Memgraph](https://docs.memgraph.com/memgraph/quick-start)

Once rsmgclient is installed, you will need to build it and then you can run
the test suite to verify it is working correctly.

```bash
cargo build
cargo test
```

## Documentation

Online documentation can be found on [docs.rs
pages](https://docs.rs/rsmgclient/).

## Code sample

Here is an example showing some of the basic commands:

```rust
use rsmgclient::{ConnectParams, Connection, MgError, Value};

fn execute_query() -> Result<(), MgError> {
    // Connect to Memgraph.
    let connect_params = ConnectParams {
        host: Some(String::from("localhost")),
        ..Default::default()
    };
    let mut connection = Connection::connect(&connect_params)?;

    // Clean existing graph.
    connection.execute_without_results("MATCH (n) DETACH DELETE n;")?;

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
