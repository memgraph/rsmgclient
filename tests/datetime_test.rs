use rsmgclient::{Connection, ConnectParams, Value};

#[test]
fn test_datetime_with_timezone() {
    // Setup: Create connection parameters and connect to the database
    let params = ConnectParams::default();
    let mut connection = Connection::connect(&params).unwrap();

    // Create a node with a datetime property including timezone
    let query = "CREATE (:Flight {AIR123: datetime({year: 2024, month: 4, day: 21, hour: 14, minute: 15, timezone: 'UTC'})})";
    connection.execute(query, None).unwrap();

    // Query the node to retrieve the datetime property
    let query = "MATCH (f:Flight) RETURN f.AIR123";
    let result = connection.execute(query, None).unwrap();

    // Extract the datetime value from the result
    if let Some(Value::Map(properties)) = result.next().unwrap() {
        if let Some(Value::DateTime(datetime)) = properties.get("AIR123") {
            // Assert the datetime fields
            assert_eq!(datetime.year, 2024);
            assert_eq!(datetime.month, 4);
            assert_eq!(datetime.day, 21);
            assert_eq!(datetime.hour, 14);
            assert_eq!(datetime.minute, 15);
            assert_eq!(datetime.second, 0);
            assert_eq!(datetime.nanosecond, 0);
            assert_eq!(datetime.time_zone_offset_seconds, 0);
            assert_eq!(datetime.time_zone_id, Some("Etc/UTC".to_string()));
        } else {
            panic!("Expected a DateTime value for AIR123");
        }
    } else {
        panic!("Expected a Map result");
    }
}
