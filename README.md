# rsmgclient - Rust Memgraph Client

[![](https://github.com/memgraph/rsmgclient/workflows/CI/badge.svg)](https://github.com/memgraph/rsmgclient/actions)

`rsmgclient` is a [Memgraph](https://memgraph.com/) database adapter for Rust
programming language. The `rsmgclient` crate is the current implementation of
the adapter. It is implemented as a wrapper around
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

Once prerequisites are met, if you want to use `rsmgclient` as a library for
your own Rust project, you can install it using `cargo`:

```bash
cargo install rsmgclient
```

NOTE: The default OpenSSL path on Windows is `C:\Program Files\OpenSSL-Win64\lib`,
if you would like to change that please provide `OPENSSL_LIB_DIR` env variable.

### Building from Source

To contribute into `rsmgclient` or just to look more closely how it is made,
you will need:

- Cloned [rsmgclient](https://github.com/memgraph/rsmgclient) repository
- Properly initialized [mgclient](https://github.com/memgraph/mgclient), please
  take care of the `mgclient` requirements.
- [Memgraph Quick Start Guide](https://memgraph.com/docs/memgraph/quick-start)

Once `rsmgclient` is cloned, you will need to build it and then you can run
the test suite to verify it is working correctly:

```bash
git submodule update --init
cargo build
# Please run Memgraph based on the quick start guide
cargo test
```

On MacOS, the build will try to detect OpenSSL by using MacPorts or Homebrew.

On Windows, `bindgen` requires `libclang` which is a part of LLVM. If LLVM is
not already installed just go to the [LLVM
download](https://releases.llvm.org/download.html) page, download and install
`LLVM.exe` file (select the option to put LLVM on the PATH). In addition,
default OpenSSL path is `C:\Program Files\OpenSSL-Win64\lib`, if you would like
to change that please provide `OPENSSL_LIB_DIR` env variable during the build
phase.

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
