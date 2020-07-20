use rsmgclient::{MgValue, connect};

fn main() {
    let connection = match connect("127.0.0.1", 7687) {
        Ok(c) => c,
        Err(err) => panic!("{}", err),
    };

    let rows: Vec<Vec<MgValue>> = match connection.execute("CREATE (n:Person {name: 'John'})-[e:KNOWS]->(m:Person {name: 'Steve'}) RETURN n, e, m;") {
        Ok(res) => res,
        Err(err) => panic!("Query failed: {}", err),
    };

    for (index, row) in rows.iter().enumerate() {
        for val in row {
            println!("{}", val);
        }
    }
}
