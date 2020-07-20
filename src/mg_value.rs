use super::bindings;
use std::ffi::CStr;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

#[derive(PartialEq)]
pub enum MgValueType {
    Null,
    Bool,
    Int,
    Float,
    String,
    List,
    Map,
    Node,
    Relationship,
    UnboundRelationship,
    Path,
    Unknown,
}

pub struct MgNode {
    pub id: i64,
    pub label_count: u32,
    pub labels: Vec<String>,
    pub properties: HashMap<String, MgValue>,
}

pub struct MgRelationship {
    pub id: i64,
    pub start_id: i64,
    pub end_id: i64,
    pub type_: String,
    pub properties: HashMap<String, MgValue>,
}

pub struct MgUnboundRelationship {
    pub id: i64,
    pub type_: String,
    pub properties: HashMap<String, MgValue>,
}

pub struct MgPath {
    pub node_count: u32,
    pub relationship_count: u32,
    pub sequence_length: u32,
    pub nodes: Vec<MgNode>,
    pub relationships: Vec<MgUnboundRelationship>,
    pub sequence: Vec<i64>,
}

pub struct MgValue {
    pub value_type: MgValueType,

    bool_value: Option<bool>,
    int_value: Option<i64>,
    float_value: Option<f64>,
    string_value: Option<String>,
    list_value: Option<Vec<MgValue>>,
    map_value: Option<HashMap<String, MgValue>>,
    node_value: Option<MgNode>,
    relationship_value: Option<MgRelationship>,
    unbound_relationship_value: Option<MgUnboundRelationship>,
    path_value: Option<MgPath>,
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

pub fn c_string_to_string(c_str: *const i8) -> String {
    let str = unsafe { CStr::from_ptr(c_str).to_str().unwrap() };
    str.to_string()
}

fn mg_string_to_string(mg_string: *const bindings::mg_string) -> String {
    let c_str = unsafe { bindings::mg_string_data(mg_string) };
    c_string_to_string(c_str)
}

fn mg_value_string(mg_value: *const bindings::mg_value) -> String {
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
        properties
    }
}

fn c_mg_unbound_relationship_to_mg_unbound_relationship(c_mg_unbound_relationship: *const bindings::mg_unbound_relationship) -> MgUnboundRelationship {
    let id = unsafe { bindings::mg_unbound_relationship_id(c_mg_unbound_relationship) };
    let type_mg_string = unsafe { bindings::mg_unbound_relationship_type(c_mg_unbound_relationship) };
    let type_ = mg_string_to_string(type_mg_string);
    let properties_mg_map = unsafe { bindings::mg_unbound_relationship_properties(c_mg_unbound_relationship) };
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

    let path_length = unsafe { bindings::mg_path_length(c_mg_path) };
    let node_count = path_length + 1;
    let relationship_count = path_length;
    let sequence_length = path_length;

    let mut nodes: Vec<MgNode> = Vec::new();
    let mut relationships: Vec<MgUnboundRelationship> = Vec::new();
    let mut sequence: Vec<i64> = Vec::new();

    for i in 0..path_length {
        let c_mg_node = unsafe { bindings::mg_path_node_at(c_mg_path, i) };
        let mg_node = c_mg_node_to_mg_node(c_mg_node);
        nodes.push(mg_node);

        let c_mg_unbound_relationship = unsafe { bindings::mg_path_relationship_at(c_mg_path, i) };
        let mg_unbound_relationship =
            c_mg_unbound_relationship_to_mg_unbound_relationship(c_mg_unbound_relationship);
        relationships.push(mg_unbound_relationship);

        sequence.push(i as i64);
    }

    MgPath {
        node_count,
        relationship_count,
        sequence_length,
        nodes,
        relationships,
        sequence,
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

impl MgValue {
    pub fn from_mg_value(c_mg_value: *const bindings::mg_value) -> MgValue {
        let mg_value_type: MgValueType =
            match unsafe { bindings::mg_value_get_type(c_mg_value) } {
                bindings::mg_value_type_MG_VALUE_TYPE_NULL => MgValueType::Null,
                bindings::mg_value_type_MG_VALUE_TYPE_BOOL => MgValueType::Bool,
                bindings::mg_value_type_MG_VALUE_TYPE_INTEGER => MgValueType::Int,
                bindings::mg_value_type_MG_VALUE_TYPE_FLOAT => MgValueType::Float,
                bindings::mg_value_type_MG_VALUE_TYPE_STRING => MgValueType::String,
                bindings::mg_value_type_MG_VALUE_TYPE_LIST => MgValueType::List,
                bindings::mg_value_type_MG_VALUE_TYPE_MAP => MgValueType::Map,
                bindings::mg_value_type_MG_VALUE_TYPE_NODE => MgValueType::Node,
                bindings::mg_value_type_MG_VALUE_TYPE_RELATIONSHIP => MgValueType::Relationship,
                bindings::mg_value_type_MG_VALUE_TYPE_UNBOUND_RELATIONSHIP => MgValueType::UnboundRelationship,
                bindings::mg_value_type_MG_VALUE_TYPE_PATH => MgValueType::Path,
                bindings::mg_value_type_MG_VALUE_TYPE_UNKNOWN => MgValueType::Unknown,
                _ => panic!("Unknown type"),
            };

        let mut mg_value = MgValue {
            value_type: MgValueType::Null,
            bool_value: None,
            int_value: None,
            float_value: None,
            string_value: None,
            list_value: None,
            map_value: None,
            node_value: None,
            relationship_value: None,
            unbound_relationship_value: None,
            path_value: None
        };

        match &mg_value_type {
            MgValueType::Null => {},
            MgValueType::Bool => mg_value.bool_value = Some(mg_value_bool(c_mg_value)),
            MgValueType::Int => mg_value.int_value = Some(mg_value_int(c_mg_value)),
            MgValueType::Float => mg_value.float_value = Some(mg_value_float(c_mg_value)),
            MgValueType::String => mg_value.string_value = Some(mg_value_string(c_mg_value)),
            MgValueType::List => mg_value.list_value = Some(mg_value_list_to_vec(c_mg_value)),
            MgValueType::Map => mg_value.map_value = Some(mg_value_map(c_mg_value)),
            MgValueType::Node => mg_value.node_value = Some(mg_value_node(c_mg_value)),
            MgValueType::Relationship => mg_value.relationship_value = Some(mg_value_relationship(c_mg_value)),
            MgValueType::UnboundRelationship => mg_value.unbound_relationship_value = Some(mg_value_unbound_relationship(c_mg_value)),
            MgValueType::Path => mg_value.path_value = Some(mg_value_path(c_mg_value)),
            MgValueType::Unknown => {},
        };

        mg_value.value_type = mg_value_type;

        mg_value
    }
}
