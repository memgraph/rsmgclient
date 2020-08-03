// Copyright (c) 2016-2020 Memgraph Ltd. [https://memgraph.com]
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
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::Formatter;


pub enum QueryParam {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<QueryParam>),
    Map(HashMap<String, QueryParam>),
}

impl QueryParam {
    fn to_c_mg_value(&self) -> *mut bindings::mg_value {
        unsafe {
            match self {
                QueryParam::Null => bindings::mg_value_make_null(),
                QueryParam::Bool(x) => bindings::mg_value_make_bool(match *x {
                    false => 0,
                    true => 1,
                }),
                QueryParam::Int(x) => bindings::mg_value_make_integer(*x),
                QueryParam::Float(x) => bindings::mg_value_make_float(*x),
                QueryParam::String(x) => bindings::mg_value_make_string(str_to_c_str(x.as_str())),
                QueryParam::List(x) => bindings::mg_value_make_list(vector_to_mg_list(x)),
                QueryParam::Map(x) => bindings::mg_value_make_map(hash_map_to_mg_map(x)),
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MgNode {
    pub id: i64,
    pub label_count: u32,
    pub labels: Vec<String>,
    pub properties: HashMap<String, MgValue>,
}

#[derive(Debug, PartialEq)]
pub struct MgRelationship {
    pub id: i64,
    pub start_id: i64,
    pub end_id: i64,
    pub type_: String,
    pub properties: HashMap<String, MgValue>,
}

#[derive(Debug, PartialEq)]
pub struct MgUnboundRelationship {
    pub id: i64,
    pub type_: String,
    pub properties: HashMap<String, MgValue>,
}

#[derive(Debug, PartialEq)]
pub struct MgPath {
    pub node_count: u32,
    pub relationship_count: u32,
    pub nodes: Vec<MgNode>,
    pub relationships: Vec<MgUnboundRelationship>,
}

#[derive(Debug, PartialEq)]
pub enum MgValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<MgValue>),
    Map(HashMap<String, MgValue>),
    Node(MgNode),
    Relationship(MgRelationship),
    UnboundRelationship(MgUnboundRelationship),
    Path(MgPath),
}

fn mg_value_list_to_vec(mg_value: *const bindings::mg_value) -> Vec<MgValue> {
    unsafe {
        let mg_list = bindings::mg_value_list(mg_value);
        mg_list_to_vec(mg_list)
    }
}

fn mg_value_bool(mg_value: *const bindings::mg_value) -> bool {
    match unsafe { bindings::mg_value_bool(mg_value) } {
        0 => false,
        _ => true,
    }
}

fn mg_value_int(mg_value: *const bindings::mg_value) -> i64 {
    unsafe { bindings::mg_value_integer(mg_value) }
}

fn mg_value_float(mg_value: *const bindings::mg_value) -> f64 {
    unsafe { bindings::mg_value_float(mg_value) }
}

pub unsafe fn c_string_to_string(c_str: *const i8) -> String {
    let str = CStr::from_ptr(c_str).to_str().unwrap();
    str.to_string()
}

fn mg_string_to_string(mg_string: *const bindings::mg_string) -> String {
    let c_str = unsafe { bindings::mg_string_data(mg_string) };
    unsafe { c_string_to_string(c_str) }
}

pub fn mg_value_string(mg_value: *const bindings::mg_value) -> String {
    let c_str = unsafe { bindings::mg_value_string(mg_value) };
    mg_string_to_string(c_str)
}

fn mg_map_to_hash_map(mg_map: *const bindings::mg_map) -> HashMap<String, MgValue> {
    unsafe {
        let size = bindings::mg_map_size(mg_map);
        let mut hash_map: HashMap<String, MgValue> = HashMap::new();
        for i in 0..size {
            let mg_string = bindings::mg_map_key_at(mg_map, i);
            let key = mg_string_to_string(mg_string);
            let map_value = bindings::mg_map_value_at(mg_map, i);
            hash_map.insert(key, MgValue::from_mg_value(map_value));
        }

        hash_map
    }
}

fn mg_value_map(mg_value: *const bindings::mg_value) -> HashMap<String, MgValue> {
    unsafe {
        let mg_map = bindings::mg_value_map(mg_value);
        mg_map_to_hash_map(mg_map)
    }
}

fn c_mg_node_to_mg_node(c_mg_node: *const bindings::mg_node) -> MgNode {
    let id = unsafe { bindings::mg_node_id(c_mg_node) };
    let label_count = unsafe { bindings::mg_node_label_count(c_mg_node) };
    let mut labels: Vec<String> = Vec::new();
    for i in 0..label_count {
        let label = unsafe { bindings::mg_node_label_at(c_mg_node, i) };
        labels.push(mg_string_to_string(label));
    }

    let properties_map = unsafe { bindings::mg_node_properties(c_mg_node) };
    let properties: HashMap<String, MgValue> = mg_map_to_hash_map(properties_map);

    MgNode {
        id,
        label_count,
        labels,
        properties,
    }
}

fn mg_value_node(mg_value: *const bindings::mg_value) -> MgNode {
    let c_mg_node = unsafe { bindings::mg_value_node(mg_value) };
    c_mg_node_to_mg_node(c_mg_node)
}

fn mg_value_relationship(mg_value: *const bindings::mg_value) -> MgRelationship {
    let c_mg_relationship = unsafe { bindings::mg_value_relationship(mg_value) };

    let id = unsafe { bindings::mg_relationship_id(c_mg_relationship) };
    let start_id = unsafe { bindings::mg_relationship_start_id(c_mg_relationship) };
    let end_id = unsafe { bindings::mg_relationship_end_id(c_mg_relationship) };
    let type_mg_string = unsafe { bindings::mg_relationship_type(c_mg_relationship) };
    let type_ = mg_string_to_string(type_mg_string);
    let properties_mg_map = unsafe { bindings::mg_relationship_properties(c_mg_relationship) };
    let properties = mg_map_to_hash_map(properties_mg_map);

    MgRelationship {
        id,
        start_id,
        end_id,
        type_,
        properties,
    }
}

fn c_mg_unbound_relationship_to_mg_unbound_relationship(
    c_mg_unbound_relationship: *const bindings::mg_unbound_relationship,
) -> MgUnboundRelationship {
    let id = unsafe { bindings::mg_unbound_relationship_id(c_mg_unbound_relationship) };
    let type_mg_string =
        unsafe { bindings::mg_unbound_relationship_type(c_mg_unbound_relationship) };
    let type_ = mg_string_to_string(type_mg_string);
    let properties_mg_map =
        unsafe { bindings::mg_unbound_relationship_properties(c_mg_unbound_relationship) };
    let properties = mg_map_to_hash_map(properties_mg_map);

    MgUnboundRelationship {
        id,
        type_,
        properties,
    }
}

fn mg_value_unbound_relationship(mg_value: *const bindings::mg_value) -> MgUnboundRelationship {
    let c_mg_unbound_relationship = unsafe { bindings::mg_value_unbound_relationship(mg_value) };
    c_mg_unbound_relationship_to_mg_unbound_relationship(c_mg_unbound_relationship)
}

fn mg_value_path(mg_value: *const bindings::mg_value) -> MgPath {
    let c_mg_path = unsafe { bindings::mg_value_path(mg_value) };
    let mut node_count = 0;
    let mut relationship_count = 0;
    let mut nodes: Vec<MgNode> = Vec::new();
    let mut relationships: Vec<MgUnboundRelationship> = Vec::new();
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
    MgPath {
        node_count,
        relationship_count,
        nodes,
        relationships,
    }
}

pub unsafe fn mg_list_to_vec(mg_list: *const bindings::mg_list) -> Vec<MgValue> {
    let size = bindings::mg_list_size(mg_list);
    let mut mg_values: Vec<MgValue> = Vec::new();
    for i in 0..size {
        let mg_value = bindings::mg_list_at(mg_list, i);
        mg_values.push(MgValue::from_mg_value(mg_value));
    }

    mg_values
}

pub fn hash_map_to_mg_map(hash_map: &HashMap<String, QueryParam>) -> *mut bindings::mg_map {
    let size = hash_map.len() as u32;
    let mg_map = unsafe { bindings::mg_map_make_empty(size) };
    for (key, val) in hash_map {
        unsafe {
            bindings::mg_map_insert(mg_map, str_to_c_str(key.as_str()), val.to_c_mg_value());
        };
    }
    mg_map
}

// allocates memory and passes ownership, user is responsible for freeing object!
pub fn str_to_c_str(string: &str) -> *const std::os::raw::c_char {
    let c_str = unsafe { Box::into_raw(Box::new(CString::new(string).unwrap())) };
    unsafe { (*c_str).as_ptr() }
}

pub fn vector_to_mg_list(vector: &Vec<QueryParam>) -> *mut bindings::mg_list {
    let size = vector.len() as u32;
    let mg_list = unsafe { bindings::mg_list_make_empty(size) };
    for mg_val in vector {
        unsafe {
            bindings::mg_list_append(mg_list, mg_val.to_c_mg_value());
        };
    }
    mg_list
}

impl MgValue {
    pub unsafe fn from_mg_value(c_mg_value: *const bindings::mg_value) -> MgValue {
        match bindings::mg_value_get_type(c_mg_value) {
            bindings::mg_value_type_MG_VALUE_TYPE_NULL => MgValue::Null,
            bindings::mg_value_type_MG_VALUE_TYPE_BOOL => MgValue::Bool(mg_value_bool(c_mg_value)),
            bindings::mg_value_type_MG_VALUE_TYPE_INTEGER => MgValue::Int(mg_value_int(c_mg_value)),
            bindings::mg_value_type_MG_VALUE_TYPE_FLOAT => {
                MgValue::Float(mg_value_float(c_mg_value))
            }
            bindings::mg_value_type_MG_VALUE_TYPE_STRING => {
                MgValue::String(mg_value_string(c_mg_value))
            }
            bindings::mg_value_type_MG_VALUE_TYPE_LIST => {
                MgValue::List(mg_value_list_to_vec(c_mg_value))
            }
            bindings::mg_value_type_MG_VALUE_TYPE_MAP => MgValue::Map(mg_value_map(c_mg_value)),
            bindings::mg_value_type_MG_VALUE_TYPE_NODE => MgValue::Node(mg_value_node(c_mg_value)),
            bindings::mg_value_type_MG_VALUE_TYPE_RELATIONSHIP => {
                MgValue::Relationship(mg_value_relationship(c_mg_value))
            }
            bindings::mg_value_type_MG_VALUE_TYPE_UNBOUND_RELATIONSHIP => {
                MgValue::UnboundRelationship(mg_value_unbound_relationship(c_mg_value))
            }
            bindings::mg_value_type_MG_VALUE_TYPE_PATH => MgValue::Path(mg_value_path(c_mg_value)),
            bindings::mg_value_type_MG_VALUE_TYPE_UNKNOWN => MgValue::Null,
            _ => panic!("Unknown type"),
        }
    }
}

impl fmt::Display for MgValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        unsafe {
            match self {
                MgValue::Null => write!(f, "NULL"),
                MgValue::Bool(x) => write!(f, "{}", x),
                MgValue::Int(x) => write!(f, "{}", x),
                MgValue::Float(x) => write!(f, "{}", x),
                MgValue::String(x) => write!(f, "'{}'", x),
                MgValue::List(x) => write!(
                    f,
                    "{}",
                    x.iter()
                        .map(|val| val.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                MgValue::Map(x) => write!(f, "{}", mg_map_to_string(x)),
                MgValue::Node(x) => write!(f, "{}", x),
                MgValue::Relationship(x) => write!(f, "{}", x),
                MgValue::UnboundRelationship(x) => write!(f, "{}", x),
                MgValue::Path(x) => write!(f, "{}", x),
            }
        }
    }
}

fn mg_map_to_string(mg_map: &HashMap<String, MgValue>) -> String {
    let mut properties: Vec<String> = Vec::new();
    let mut sorted: Vec<_> = mg_map.iter().collect();
    sorted.sort_by(|x, y| x.0.cmp(&y.0));
    for (key, value) in sorted {
        properties.push(format!("'{}': {}", key, value));
    }
    return format!("{{{}}}", properties.join(", "));
}

impl fmt::Display for MgNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(:{} {})",
            self.labels.join(", "),
            mg_map_to_string(&self.properties)
        )
    }
}

impl fmt::Display for MgRelationship {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[:{} {}]",
            self.type_,
            mg_map_to_string(&self.properties)
        )
    }
}

impl fmt::Display for MgUnboundRelationship {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[:{} {}]",
            self.type_,
            mg_map_to_string(&self.properties)
        )
    }
}

impl fmt::Display for MgPath {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests;
