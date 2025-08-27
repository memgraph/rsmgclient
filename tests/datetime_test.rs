use rsmgclient::{ConnectParams, Connection, Value};

#[test]
fn test_datetime_with_timezone() {
    // Setup: Create connection parameters and connect to the database
    let params = ConnectParams {
        host: Some(String::from("localhost")),
        ..ConnectParams::default()
    };
    let mut connection = Connection::connect(&params).unwrap();

    // Create a node with a datetime property including timezone
    let query = "CREATE (:Flight {AIR123: datetime({year: 2024, month: 4, day: 21, hour: 14, minute: 15, timezone: 'UTC'})})";
    connection.execute(query, None).unwrap();
    connection.fetchall().unwrap(); // Complete the first query

    // Query the node to retrieve the datetime property
    let query = "MATCH (f:Flight) RETURN f.AIR123";
    connection.execute(query, None).unwrap();
    let records = connection.fetchall().unwrap();

    // Extract the datetime value from the result
    if let Some(record) = records.first() {
        if let Some(Value::DateTime(datetime)) = record.values.get(0) {
            // Assert the datetime fields
            assert_eq!(datetime.year, 2024);
            assert_eq!(datetime.month, 4);
            assert_eq!(datetime.day, 21);
            assert_eq!(datetime.hour, 14);
            assert_eq!(datetime.minute, 15);
            assert_eq!(datetime.second, 0);
            assert_eq!(datetime.nanosecond, 0);
            assert_eq!(datetime.time_zone_offset_seconds, 0);
            // Check that timezone ID is either "Etc/UTC" or a system-specific UTC representation
            assert!(
                datetime.time_zone_id == Some("Etc/UTC".to_string())
                    || datetime
                        .time_zone_id
                        .as_ref()
                        .map_or(false, |id| id.starts_with("TZ_")),
                "Expected timezone ID to be 'Etc/UTC' or start with 'TZ_', got {:?}",
                datetime.time_zone_id
            );
        } else {
            panic!("Expected a DateTime value for AIR123");
        }
    } else {
        panic!("Expected at least one record");
    }
}
