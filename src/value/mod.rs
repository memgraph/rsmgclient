// Copyright (c) 2016-2022 Memgraph Ltd. [https://memgraph.com]
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

use super::bindings;
use chrono::{
    Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Timelike, Utc,
};
use chrono_tz::Tz;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::Formatter;
use std::num::TryFromIntError;
use std::os::raw::c_char;
use std::slice;

/// Representation of Point2D spatial data type.
#[derive(Debug, PartialEq, Clone)]
pub struct Point2D {
    pub srid: u16,
    pub x_longitude: f64,
    pub y_latitude: f64,
}

impl fmt::Display for Point2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Point2D({{ srid:{}, x:{}, y:{} }})",
            self.srid, self.x_longitude, self.y_latitude
        )
    }
}

/// Representation of Point3D spatial data type.
#[derive(Debug, PartialEq, Clone)]
pub struct Point3D {
    pub srid: u16,
    pub x_longitude: f64,
    pub y_latitude: f64,
    pub z_height: f64,
}

impl fmt::Display for Point3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Point3D({{ srid:{}, x:{}, y:{}, z:{} }})",
            self.srid, self.x_longitude, self.y_latitude, self.z_height
        )
    }
}

/// Representation of parameter value used in query.
pub enum QueryParam {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Date(NaiveDate),
    LocalTime(NaiveTime),
    LocalDateTime(NaiveDateTime),
    Duration(Duration),
    Point2D(Point2D),
    Point3D(Point3D),
    List(Vec<QueryParam>),
    Map(HashMap<String, QueryParam>),
}

impl QueryParam {
    fn to_c_mg_value(&self) -> *mut bindings::mg_value {
        unsafe {
            match self {
                QueryParam::Null => bindings::mg_value_make_null(),
                QueryParam::Bool(x) => {
                    let ptr = bindings::mg_value_make_bool(match *x {
                        false => 0,
                        true => 1,
                    });
                    if ptr.is_null() {
                        return bindings::mg_value_make_null();
                    }
                    ptr
                }
                QueryParam::Int(x) => {
                    let ptr = bindings::mg_value_make_integer(*x);
                    if ptr.is_null() {
                        return bindings::mg_value_make_null();
                    }
                    ptr
                }
                QueryParam::Float(x) => {
                    let ptr = bindings::mg_value_make_float(*x);
                    if ptr.is_null() {
                        return bindings::mg_value_make_null();
                    }
                    ptr
                }
                QueryParam::String(x) => {
                    // String parameter may contain null bytes - return null on error
                    let c_string = match CString::new(x.as_str()) {
                        Ok(s) => s,
                        Err(_) => return bindings::mg_value_make_null(),
                    };
                    let ptr = bindings::mg_value_make_string(c_string.as_ptr());
                    if ptr.is_null() {
                        return bindings::mg_value_make_null();
                    }
                    ptr
                }
                QueryParam::Date(x) => {
                    let mg_date = naive_date_to_mg_date(x);
                    if mg_date.is_null() {
                        return bindings::mg_value_make_null();
                    }
                    let ptr = bindings::mg_value_make_date(mg_date);
                    if ptr.is_null() {
                        bindings::mg_date_destroy(mg_date);
                        return bindings::mg_value_make_null();
                    }
                    ptr
                }
                QueryParam::LocalTime(x) => {
                    let mg_local_time = naive_local_time_to_mg_local_time(x);
                    if mg_local_time.is_null() {
                        return bindings::mg_value_make_null();
                    }
                    let ptr = bindings::mg_value_make_local_time(mg_local_time);
                    if ptr.is_null() {
                        bindings::mg_local_time_destroy(mg_local_time);
                        return bindings::mg_value_make_null();
                    }
                    ptr
                }
                QueryParam::LocalDateTime(x) => {
                    let mg_local_date_time = naive_local_date_time_to_mg_local_date_time(x);
                    if mg_local_date_time.is_null() {
                        return bindings::mg_value_make_null();
                    }
                    let ptr = bindings::mg_value_make_local_date_time(mg_local_date_time);
                    if ptr.is_null() {
                        bindings::mg_local_date_time_destroy(mg_local_date_time);
                        return bindings::mg_value_make_null();
                    }
                    ptr
                }
                QueryParam::Duration(x) => {
                    let mg_duration = duration_to_mg_duration(x);
                    if mg_duration.is_null() {
                        return bindings::mg_value_make_null();
                    }
                    let ptr = bindings::mg_value_make_duration(mg_duration);
                    if ptr.is_null() {
                        bindings::mg_duration_destroy(mg_duration);
                        return bindings::mg_value_make_null();
                    }
                    ptr
                }
                QueryParam::Point2D(x) => {
                    let mg_point_2d = point2d_to_mg_point_2d(x);
                    if mg_point_2d.is_null() {
                        return bindings::mg_value_make_null();
                    }
                    let ptr = bindings::mg_value_make_point_2d(mg_point_2d);
                    if ptr.is_null() {
                        bindings::mg_point_2d_destroy(mg_point_2d);
                        return bindings::mg_value_make_null();
                    }
                    ptr
                }
                QueryParam::Point3D(x) => {
                    let mg_point_3d = point3d_to_mg_point_3d(x);
                    if mg_point_3d.is_null() {
                        return bindings::mg_value_make_null();
                    }
                    let ptr = bindings::mg_value_make_point_3d(mg_point_3d);
                    if ptr.is_null() {
                        bindings::mg_point_3d_destroy(mg_point_3d);
                        return bindings::mg_value_make_null();
                    }
                    ptr
                }
                QueryParam::List(x) => {
                    let mg_list = vector_to_mg_list(x);
                    if mg_list.is_null() {
                        return bindings::mg_value_make_null();
                    }
                    let ptr = bindings::mg_value_make_list(mg_list);
                    if ptr.is_null() {
                        bindings::mg_list_destroy(mg_list);
                        return bindings::mg_value_make_null();
                    }
                    ptr
                }
                QueryParam::Map(x) => {
                    let mg_map = hash_map_to_mg_map(x);
                    if mg_map.is_null() {
                        return bindings::mg_value_make_null();
                    }
                    let ptr = bindings::mg_value_make_map(mg_map);
                    if ptr.is_null() {
                        bindings::mg_map_destroy(mg_map);
                        return bindings::mg_value_make_null();
                    }
                    ptr
                }
            }
        }
    }
}

/// Representation of a DateTime value with timezone support.
///
/// Contains date, time, and timezone information including timezone ID and offset.
#[derive(Debug, PartialEq, Clone)]
pub struct DateTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub nanosecond: u32,
    pub time_zone_offset_seconds: i32,
    pub time_zone_id: Option<String>,
}

/// Representation of node value from a labeled property graph.
///
/// Consists of a unique identifier(within the scope of its origin graph), a list
/// of labels and a map of properties.
///
/// Maximum possible number of labels allowed by Bolt protocol is UINT32_MAX
#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    pub id: i64,
    pub label_count: u32,
    pub labels: Vec<String>,
    pub properties: HashMap<String, Value>,
}

/// Representation of relationship value from a labeled property graph.
///
/// Consists of a unique identifier(within the scope of its origin graph),
/// identifiers for the start and end nodes of that relationship, a type and
/// a map of properties.
#[derive(Debug, PartialEq, Clone)]
pub struct Relationship {
    pub id: i64,
    pub start_id: i64,
    pub end_id: i64,
    pub type_: String,
    pub properties: HashMap<String, Value>,
}

/// Representation of relationship from a labeled property graph.
///
/// Relationship without start and end nodes. Mainly used as a supporting type
/// for Path.
#[derive(Debug, PartialEq, Clone)]
pub struct UnboundRelationship {
    pub id: i64,
    pub type_: String,
    pub properties: HashMap<String, Value>,
}

/// Representation of sequence of alternating nodes and relationships corresponding
/// to a walk in a labeled property graph.
///
/// Path always begins and ends with a node. It consists of a list of distinct
/// nodes, a list of distinct relationships and a sequence of integers
/// describing the path traversal.
#[derive(Debug, PartialEq, Clone)]
pub struct Path {
    pub node_count: u32,
    pub relationship_count: u32,
    pub nodes: Vec<Node>,
    pub relationships: Vec<UnboundRelationship>,
}

/// Representation of Bolt value returned by database.
///
/// Value is can be any of the types specified by Bolt protocol.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Date(NaiveDate),
    LocalTime(NaiveTime),
    LocalDateTime(NaiveDateTime),
    DateTime(DateTime),
    Duration(Duration),
    Point2D(Point2D),
    Point3D(Point3D),
    Map(HashMap<String, Value>),
    Node(Node),
    Relationship(Relationship),
    UnboundRelationship(UnboundRelationship),
    Path(Path),
}

/// Representation of a single row returned by database.
pub struct Record {
    pub values: Vec<Value>,
}

fn mg_value_list_to_vec(mg_value: *const bindings::mg_value) -> Vec<Value> {
    unsafe {
        let mg_list = bindings::mg_value_list(mg_value);
        mg_list_to_vec(mg_list)
    }
}

fn mg_value_bool(mg_value: *const bindings::mg_value) -> bool {
    !matches!(unsafe { bindings::mg_value_bool(mg_value) }, 0)
}

fn mg_value_int(mg_value: *const bindings::mg_value) -> i64 {
    unsafe { bindings::mg_value_integer(mg_value) }
}

fn mg_value_float(mg_value: *const bindings::mg_value) -> f64 {
    unsafe { bindings::mg_value_float(mg_value) }
}

pub(crate) unsafe fn c_string_to_string(c_str: *const c_char, size: Option<u32>) -> String {
    // https://github.com/rust-lang/rust/blob/master/library/std/src/ffi/c_str.rs#L1230
    let c_str = match size {
        Some(x) => unsafe {
            CStr::from_bytes_with_nul_unchecked(slice::from_raw_parts(
                c_str as *const u8,
                (x + 1) as usize,
            ))
        },
        None => unsafe { CStr::from_ptr(c_str) },
    };
    // Convert to string, using lossy conversion if UTF-8 is invalid
    c_str.to_str().unwrap_or("").to_string()
}

fn mg_string_to_string(mg_string: *const bindings::mg_string) -> String {
    let c_str = unsafe { bindings::mg_string_data(mg_string) };
    unsafe { c_string_to_string(c_str, Some(bindings::mg_string_size(mg_string))) }
}

pub(crate) fn mg_value_string(mg_value: *const bindings::mg_value) -> String {
    let c_str = unsafe { bindings::mg_value_string(mg_value) };
    mg_string_to_string(c_str)
}

fn days_as_seconds(days: i64) -> i64 {
    hours_as_seconds(days * 24)
}

fn hours_as_seconds(hours: i64) -> i64 {
    minutes_as_seconds(hours * 60)
}

fn minutes_as_seconds(minutes: i64) -> i64 {
    minutes * 60
}

const NSEC_IN_SEC: i64 = 1_000_000_000;

pub(crate) fn mg_value_naive_date(mg_value: *const bindings::mg_value) -> Result<NaiveDate, ()> {
    let c_date = unsafe { bindings::mg_value_date(mg_value) };
    let c_delta_days = unsafe { bindings::mg_date_days(c_date) };
    // Unix epoch date is a known valid date
    let epoch_date = NaiveDate::from_ymd_opt(1970, 1, 1).expect("Unix epoch is a valid date");
    let delta_days = Duration::days(c_delta_days);
    epoch_date.checked_add_signed(delta_days).ok_or(())
}

pub(crate) fn mg_value_naive_local_time(
    mg_value: *const bindings::mg_value,
) -> Result<NaiveTime, TryFromIntError> {
    let c_local_time = unsafe { bindings::mg_value_local_time(mg_value) };
    let c_nanoseconds = unsafe { bindings::mg_local_time_nanoseconds(c_local_time) };
    let seconds = u32::try_from(c_nanoseconds / NSEC_IN_SEC)?;
    let nanoseconds = u32::try_from(c_nanoseconds % NSEC_IN_SEC)?;
    NaiveTime::from_num_seconds_from_midnight_opt(seconds, nanoseconds)
        .ok_or_else(|| u32::try_from(-1).unwrap_err())
}

pub(crate) fn mg_value_naive_local_date_time(
    mg_value: *const bindings::mg_value,
) -> Result<NaiveDateTime, TryFromIntError> {
    let c_local_date_time = unsafe { bindings::mg_value_local_date_time(mg_value) };
    let c_seconds = unsafe { bindings::mg_local_date_time_seconds(c_local_date_time) };
    let c_nanoseconds = unsafe { bindings::mg_local_date_time_nanoseconds(c_local_date_time) };
    let nanoseconds = u32::try_from(c_nanoseconds)?;
    Utc.timestamp_opt(c_seconds, nanoseconds)
        .single()
        .map(|dt| dt.naive_utc())
        .ok_or_else(|| u32::try_from(-1).unwrap_err())
}

fn mg_value_datetime_zone_id(
    c_datetime_zone_id: *const bindings::mg_date_time_zone_id,
) -> Result<DateTime, crate::error::MgError> {
    let c_seconds = unsafe { bindings::mg_date_time_zone_id_seconds(c_datetime_zone_id) };
    let c_nanoseconds = unsafe { bindings::mg_date_time_zone_id_nanoseconds(c_datetime_zone_id) };
    let c_timezone_name_ptr =
        unsafe { bindings::mg_date_time_zone_id_timezone_name(c_datetime_zone_id) };

    // Create NaiveDateTime from timestamp
    let naive_datetime = match Utc.timestamp_opt(c_seconds, c_nanoseconds as u32) {
        chrono::LocalResult::Single(dt) => dt.naive_utc(),
        _ => {
            return Err(crate::error::MgError::InvalidTimestamp);
        }
    };

    // Extract timezone name from mg_string
    let timezone_name = if c_timezone_name_ptr.is_null() {
        "UTC".to_string()
    } else {
        mg_string_to_string(c_timezone_name_ptr)
    };

    // Compute timezone offset from timezone name and timestamp
    let time_zone_offset_seconds = timezone_name
        .parse::<Tz>()
        .ok()
        .and_then(|tz| {
            // Convert UTC timestamp to the specified timezone
            let utc_datetime = Utc
                .timestamp_opt(c_seconds, c_nanoseconds as u32)
                .single()?;
            let zoned_datetime = utc_datetime.with_timezone(&tz);
            Some(zoned_datetime.offset().fix().local_minus_utc())
        })
        .unwrap_or(0); // Default to 0 if timezone parsing fails

    // Extract individual date/time fields
    let date = naive_datetime.date();
    let time = naive_datetime.time();

    Ok(DateTime {
        year: date.year(),
        month: date.month(),
        day: date.day(),
        hour: time.hour(),
        minute: time.minute(),
        second: time.second(),
        nanosecond: time.nanosecond(),
        time_zone_offset_seconds,
        time_zone_id: Some(timezone_name),
    })
}

pub(crate) fn mg_value_duration(mg_value: *const bindings::mg_value) -> Duration {
    let c_duration = unsafe { bindings::mg_value_duration(mg_value) };
    let days = unsafe { bindings::mg_duration_days(c_duration) };
    let seconds = unsafe { bindings::mg_duration_seconds(c_duration) };
    let nanoseconds = unsafe { bindings::mg_duration_nanoseconds(c_duration) };
    Duration::days(days) + Duration::seconds(seconds) + Duration::nanoseconds(nanoseconds)
}

pub(crate) fn mg_value_point2d(mg_value: *const bindings::mg_value) -> Point2D {
    let c_point2d = unsafe { bindings::mg_value_point_2d(mg_value) };
    let srid = unsafe { bindings::mg_point_2d_srid(c_point2d) } as u16;
    let x_longitude = unsafe { bindings::mg_point_2d_x(c_point2d) };
    let y_latitude = unsafe { bindings::mg_point_2d_y(c_point2d) };
    Point2D {
        srid,
        x_longitude,
        y_latitude,
    }
}

pub(crate) fn mg_value_point3d(mg_value: *const bindings::mg_value) -> Point3D {
    let c_point3d = unsafe { bindings::mg_value_point_3d(mg_value) };
    let srid = unsafe { bindings::mg_point_3d_srid(c_point3d) } as u16;
    let x_longitude = unsafe { bindings::mg_point_3d_x(c_point3d) };
    let y_latitude = unsafe { bindings::mg_point_3d_y(c_point3d) };
    let z_height = unsafe { bindings::mg_point_3d_z(c_point3d) };
    Point3D {
        srid,
        x_longitude,
        y_latitude,
        z_height,
    }
}

pub(crate) fn mg_map_to_hash_map(mg_map: *const bindings::mg_map) -> HashMap<String, Value> {
    unsafe {
        let size = bindings::mg_map_size(mg_map);
        let mut hash_map: HashMap<String, Value> = HashMap::new();
        for i in 0..size {
            let mg_string = bindings::mg_map_key_at(mg_map, i);
            let key = mg_string_to_string(mg_string);
            let map_value = bindings::mg_map_value_at(mg_map, i);
            hash_map.insert(key, Value::from_mg_value(map_value));
        }

        hash_map
    }
}

fn mg_value_map(mg_value: *const bindings::mg_value) -> HashMap<String, Value> {
    unsafe {
        let mg_map = bindings::mg_value_map(mg_value);
        mg_map_to_hash_map(mg_map)
    }
}

fn c_mg_node_to_mg_node(c_mg_node: *const bindings::mg_node) -> Node {
    let id = unsafe { bindings::mg_node_id(c_mg_node) };
    let label_count = unsafe { bindings::mg_node_label_count(c_mg_node) };
    let mut labels: Vec<String> = Vec::new();
    for i in 0..label_count {
        let label = unsafe { bindings::mg_node_label_at(c_mg_node, i) };
        labels.push(mg_string_to_string(label));
    }

    let properties_map = unsafe { bindings::mg_node_properties(c_mg_node) };
    let properties: HashMap<String, Value> = mg_map_to_hash_map(properties_map);

    Node {
        id,
        label_count,
        labels,
        properties,
    }
}

fn mg_value_node(mg_value: *const bindings::mg_value) -> Node {
    let c_mg_node = unsafe { bindings::mg_value_node(mg_value) };
    c_mg_node_to_mg_node(c_mg_node)
}

fn mg_value_relationship(mg_value: *const bindings::mg_value) -> Relationship {
    let c_mg_relationship = unsafe { bindings::mg_value_relationship(mg_value) };

    let id = unsafe { bindings::mg_relationship_id(c_mg_relationship) };
    let start_id = unsafe { bindings::mg_relationship_start_id(c_mg_relationship) };
    let end_id = unsafe { bindings::mg_relationship_end_id(c_mg_relationship) };
    let type_mg_string = unsafe { bindings::mg_relationship_type(c_mg_relationship) };
    let type_ = mg_string_to_string(type_mg_string);
    let properties_mg_map = unsafe { bindings::mg_relationship_properties(c_mg_relationship) };
    let properties = mg_map_to_hash_map(properties_mg_map);

    Relationship {
        id,
        start_id,
        end_id,
        type_,
        properties,
    }
}

fn c_mg_unbound_relationship_to_mg_unbound_relationship(
    c_mg_unbound_relationship: *const bindings::mg_unbound_relationship,
) -> UnboundRelationship {
    let id = unsafe { bindings::mg_unbound_relationship_id(c_mg_unbound_relationship) };
    let type_mg_string =
        unsafe { bindings::mg_unbound_relationship_type(c_mg_unbound_relationship) };
    let type_ = mg_string_to_string(type_mg_string);
    let properties_mg_map =
        unsafe { bindings::mg_unbound_relationship_properties(c_mg_unbound_relationship) };
    let properties = mg_map_to_hash_map(properties_mg_map);

    UnboundRelationship {
        id,
        type_,
        properties,
    }
}

fn mg_value_unbound_relationship(mg_value: *const bindings::mg_value) -> UnboundRelationship {
    let c_mg_unbound_relationship = unsafe { bindings::mg_value_unbound_relationship(mg_value) };
    c_mg_unbound_relationship_to_mg_unbound_relationship(c_mg_unbound_relationship)
}

fn mg_value_path(mg_value: *const bindings::mg_value) -> Path {
    let c_mg_path = unsafe { bindings::mg_value_path(mg_value) };
    let mut node_count = 0;
    let mut relationship_count = 0;
    let mut nodes: Vec<Node> = Vec::new();
    let mut relationships: Vec<UnboundRelationship> = Vec::new();
    loop {
        let c_mg_node = unsafe { bindings::mg_path_node_at(c_mg_path, node_count) };
        if c_mg_node.is_null() {
            break;
        }
        node_count += 1;
        nodes.push(c_mg_node_to_mg_node(c_mg_node));
    }
    loop {
        let c_mg_unbound_relationship =
            unsafe { bindings::mg_path_relationship_at(c_mg_path, relationship_count) };
        if c_mg_unbound_relationship.is_null() {
            break;
        }
        relationship_count += 1;
        relationships.push(c_mg_unbound_relationship_to_mg_unbound_relationship(
            c_mg_unbound_relationship,
        ));
    }
    Path {
        node_count,
        relationship_count,
        nodes,
        relationships,
    }
}

pub(crate) unsafe fn mg_list_to_vec(mg_list: *const bindings::mg_list) -> Vec<Value> {
    let size = unsafe { bindings::mg_list_size(mg_list) };
    let mut mg_values: Vec<Value> = Vec::new();
    for i in 0..size {
        let mg_value = unsafe { bindings::mg_list_at(mg_list, i) };
        mg_values.push(unsafe { Value::from_mg_value(mg_value) });
    }

    mg_values
}

pub(crate) fn hash_map_to_mg_map(hash_map: &HashMap<String, QueryParam>) -> *mut bindings::mg_map {
    let size = hash_map.len() as u32;
    let mg_map = unsafe { bindings::mg_map_make_empty(size) };
    if mg_map.is_null() {
        return std::ptr::null_mut();
    }
    for (key, val) in hash_map {
        // Skip keys with null bytes - they cannot be converted to C strings
        if let Ok(c_key) = CString::new(key.as_str()) {
            let mg_value = val.to_c_mg_value();
            if mg_value.is_null() {
                // If value allocation fails, destroy the map and return null
                unsafe { bindings::mg_map_destroy(mg_map) };
                return std::ptr::null_mut();
            }
            unsafe {
                bindings::mg_map_insert(mg_map, c_key.as_ptr(), mg_value);
            };
        }
    }
    mg_map
}

pub(crate) fn naive_date_to_mg_date(input: &NaiveDate) -> *mut bindings::mg_date {
    // Unix epoch date is a known valid date
    let unix_epoch = NaiveDate::from_ymd_opt(1970, 1, 1)
        .expect("Unix epoch is a valid date")
        .num_days_from_ce();
    let ptr = unsafe { bindings::mg_date_make((input.num_days_from_ce() - unix_epoch) as i64) };
    // mg_date_make can return NULL on OOM
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    ptr
}

pub(crate) fn naive_local_time_to_mg_local_time(input: &NaiveTime) -> *mut bindings::mg_local_time {
    let hours_ns = hours_as_seconds(input.hour() as i64) * NSEC_IN_SEC;
    let minutes_ns = minutes_as_seconds(input.minute() as i64) * NSEC_IN_SEC;
    let seconds_ns = (input.second() as i64) * NSEC_IN_SEC;
    let nanoseconds = input.nanosecond() as i64;
    let ptr =
        unsafe { bindings::mg_local_time_make(hours_ns + minutes_ns + seconds_ns + nanoseconds) };
    // mg_local_time_make can return NULL on OOM
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    ptr
}

pub(crate) fn naive_local_date_time_to_mg_local_date_time(
    input: &NaiveDateTime,
) -> *mut bindings::mg_local_date_time {
    // Unix epoch date is a known valid date
    let unix_epoch = NaiveDate::from_ymd_opt(1970, 1, 1)
        .expect("Unix epoch is a valid date")
        .num_days_from_ce();
    let days_s = days_as_seconds((input.num_days_from_ce() - unix_epoch) as i64);
    let hours_s = hours_as_seconds(input.hour() as i64);
    let minutes_s = minutes_as_seconds(input.minute() as i64);
    let seconds_s = input.second() as i64;
    let nanoseconds = input.nanosecond() as i64;
    let ptr = unsafe {
        bindings::mg_local_date_time_make(days_s + hours_s + minutes_s + seconds_s, nanoseconds)
    };
    // mg_local_date_time_make can return NULL on OOM
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    ptr
}

pub(crate) fn duration_to_mg_duration(input: &Duration) -> *mut bindings::mg_duration {
    // Duration returns total number of nanoseconds, in order to create a valid mg_duration object,
    // days and seconds have to be reducted from the total duration. In addition, one can get numer
    // of nanoseconds and then substract days and seconds, but since nanoseconds can overflow quite
    // quicky (with 292 years), it's better to use Duration and first reduce days and seconds.
    let mut duration = *input;
    let days = input.num_days();
    duration -= Duration::days(days);
    let seconds = input.num_seconds();
    duration -= Duration::seconds(seconds);
    // After subtracting days and seconds, remaining nanoseconds should always fit in i64
    let nanoseconds = duration.num_nanoseconds().unwrap_or(0);
    let ptr = unsafe { bindings::mg_duration_make(0, days, seconds, nanoseconds) };
    // mg_duration_make can return NULL on OOM
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    ptr
}

pub(crate) fn point2d_to_mg_point_2d(input: &Point2D) -> *mut bindings::mg_point_2d {
    let ptr =
        unsafe { bindings::mg_point_2d_make(input.srid, input.x_longitude, input.y_latitude) };
    // mg_point_2d_make can return NULL on OOM
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    ptr
}

pub(crate) fn point3d_to_mg_point_3d(input: &Point3D) -> *mut bindings::mg_point_3d {
    let ptr = unsafe {
        bindings::mg_point_3d_make(
            input.srid,
            input.x_longitude,
            input.y_latitude,
            input.z_height,
        )
    };
    // mg_point_3d_make can return NULL on OOM
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    ptr
}

pub(crate) fn vector_to_mg_list(vector: &[QueryParam]) -> *mut bindings::mg_list {
    let size = vector.len() as u32;
    let mg_list = unsafe { bindings::mg_list_make_empty(size) };
    if mg_list.is_null() {
        return std::ptr::null_mut();
    }
    for mg_val in vector {
        let mg_value = mg_val.to_c_mg_value();
        if mg_value.is_null() {
            // If value allocation fails, destroy the list and return null
            unsafe { bindings::mg_list_destroy(mg_list) };
            return std::ptr::null_mut();
        }
        unsafe {
            bindings::mg_list_append(mg_list, mg_value);
        };
    }
    mg_list
}

impl Value {
    pub(crate) unsafe fn from_mg_value(c_mg_value: *const bindings::mg_value) -> Value {
        match unsafe { bindings::mg_value_get_type(c_mg_value) } {
            bindings::mg_value_type_MG_VALUE_TYPE_NULL => Value::Null,
            bindings::mg_value_type_MG_VALUE_TYPE_BOOL => Value::Bool(mg_value_bool(c_mg_value)),
            bindings::mg_value_type_MG_VALUE_TYPE_INTEGER => Value::Int(mg_value_int(c_mg_value)),
            bindings::mg_value_type_MG_VALUE_TYPE_FLOAT => Value::Float(mg_value_float(c_mg_value)),
            bindings::mg_value_type_MG_VALUE_TYPE_STRING => {
                Value::String(mg_value_string(c_mg_value))
            }
            bindings::mg_value_type_MG_VALUE_TYPE_DATE => {
                // If date conversion fails, return Null instead of panicking
                mg_value_naive_date(c_mg_value)
                    .map(Value::Date)
                    .unwrap_or(Value::Null)
            }
            bindings::mg_value_type_MG_VALUE_TYPE_LOCAL_TIME => {
                // If time conversion fails, return Null instead of panicking
                mg_value_naive_local_time(c_mg_value)
                    .map(Value::LocalTime)
                    .unwrap_or(Value::Null)
            }
            bindings::mg_value_type_MG_VALUE_TYPE_LOCAL_DATE_TIME => {
                // If datetime conversion fails, return Null instead of panicking
                mg_value_naive_local_date_time(c_mg_value)
                    .map(Value::LocalDateTime)
                    .unwrap_or(Value::Null)
            }
            bindings::mg_value_type_MG_VALUE_TYPE_DATE_TIME_ZONE_ID => {
                let c_datetime_zone_id =
                    unsafe { bindings::mg_value_date_time_zone_id(c_mg_value) };
                // If datetime conversion fails, return Null instead of panicking
                mg_value_datetime_zone_id(c_datetime_zone_id)
                    .map(Value::DateTime)
                    .unwrap_or(Value::Null)
            }
            bindings::mg_value_type_MG_VALUE_TYPE_DURATION => {
                Value::Duration(mg_value_duration(c_mg_value))
            }
            bindings::mg_value_type_MG_VALUE_TYPE_POINT_2D => {
                Value::Point2D(mg_value_point2d(c_mg_value))
            }
            bindings::mg_value_type_MG_VALUE_TYPE_POINT_3D => {
                Value::Point3D(mg_value_point3d(c_mg_value))
            }
            bindings::mg_value_type_MG_VALUE_TYPE_LIST => {
                Value::List(mg_value_list_to_vec(c_mg_value))
            }
            bindings::mg_value_type_MG_VALUE_TYPE_MAP => Value::Map(mg_value_map(c_mg_value)),
            bindings::mg_value_type_MG_VALUE_TYPE_NODE => Value::Node(mg_value_node(c_mg_value)),
            bindings::mg_value_type_MG_VALUE_TYPE_RELATIONSHIP => {
                Value::Relationship(mg_value_relationship(c_mg_value))
            }
            bindings::mg_value_type_MG_VALUE_TYPE_UNBOUND_RELATIONSHIP => {
                Value::UnboundRelationship(mg_value_unbound_relationship(c_mg_value))
            }
            bindings::mg_value_type_MG_VALUE_TYPE_PATH => Value::Path(mg_value_path(c_mg_value)),
            bindings::mg_value_type_MG_VALUE_TYPE_UNKNOWN => Value::Null,
            _ => panic!("Unknown type"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Bool(x) => write!(f, "{}", x),
            Value::Int(x) => write!(f, "{}", x),
            Value::Float(x) => write!(f, "{}", x),
            Value::String(x) => write!(f, "'{}'", x),
            Value::Date(x) => write!(f, "'{}'", x),
            Value::LocalTime(x) => write!(f, "'{}'", x),
            Value::LocalDateTime(x) => write!(f, "'{}'", x),
            Value::DateTime(x) => write!(
                f,
                "'{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:09} {}{:02}:{:02}'",
                x.year,
                x.month,
                x.day,
                x.hour,
                x.minute,
                x.second,
                x.nanosecond,
                if x.time_zone_offset_seconds >= 0 {
                    "+"
                } else {
                    "-"
                },
                x.time_zone_offset_seconds.abs() / 3600,
                (x.time_zone_offset_seconds.abs() % 3600) / 60
            ),
            Value::Duration(x) => write!(f, "'{}'", x),
            Value::Point2D(x) => write!(f, "'{}'", x),
            Value::Point3D(x) => write!(f, "'{}'", x),
            Value::List(x) => write!(
                f,
                "{}",
                x.iter()
                    .map(|val| val.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::Map(x) => write!(f, "{}", mg_map_to_string(x)),
            Value::Node(x) => write!(f, "{}", x),
            Value::Relationship(x) => write!(f, "{}", x),
            Value::UnboundRelationship(x) => write!(f, "{}", x),
            Value::Path(x) => write!(f, "{}", x),
        }
    }
}

fn mg_map_to_string(mg_map: &HashMap<String, Value>) -> String {
    let mut properties: Vec<String> = Vec::new();
    let mut sorted: Vec<_> = mg_map.iter().collect();
    sorted.sort_by(|x, y| x.0.cmp(y.0));
    for (key, value) in sorted {
        properties.push(format!("'{}': {}", key, value));
    }
    format!("{{{}}}", properties.join(", "))
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(:{} {})",
            self.labels.join(", "),
            mg_map_to_string(&self.properties)
        )
    }
}

impl fmt::Display for Relationship {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[:{} {}]",
            self.type_,
            mg_map_to_string(&self.properties)
        )
    }
}

impl fmt::Display for UnboundRelationship {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[:{} {}]",
            self.type_,
            mg_map_to_string(&self.properties)
        )
    }
}

impl fmt::Display for Path {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests;
