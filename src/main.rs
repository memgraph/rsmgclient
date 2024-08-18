use std::collections::HashMap;

use rsmgclient::{ConnectParams, Connection, MgError, Point2D, QueryParam, Value};

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

    let mut query_params: HashMap<String, QueryParam> = HashMap::new();
    query_params.insert(
        "point2d".to_string(),
        QueryParam::Point2D(Point2D {
            srid: 7203,
            x_longitude: 0.0,
            y_latitude: 1.0,
        }),
    );
    connection.execute("RETURN $point2d;", Some(&query_params))?;
    for record in connection.fetchall()? {
        for value in record.values {
            println!("{}", value);
        }
    }

    Ok(())
}

fn main() {
    if let Err(error) = execute_query() {
        panic!("{}", error)
    }
}
