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
    pub fn get_bool_value(&self) -> bool {
        if self.value_type != MgValueType::Bool {
            panic!("Not bool value");
        }
        match self.bool_value {
            Some(x) => x,
            None => panic!("Not bool value"),
        }
    }

    pub fn get_int_value(&self) -> i64 {
        if self.value_type != MgValueType::Int {
            panic!("Not int value");
        }
        match self.int_value {
            Some(x) => x,
            None => panic!("Not int value"),
        }
    }

    pub fn get_float_value(&self) -> f64 {
        if self.value_type != MgValueType::Float {
            panic!("Not float value");
        }
        match self.float_value {
            Some(x) => x,
            None => panic!("Not float value"),
        }
    }

    pub fn get_string_value(&self) -> &String {
        if self.value_type != MgValueType::String {
            panic!("Not String value");
        }
        match &self.string_value {
            Some(x) => &x,
            None => panic!("Not string value"),
        }
    }

    pub fn get_list_value(&self) -> &Vec<MgValue> {
        if self.value_type != MgValueType::List {
            panic!("Not list value");
        }
        match &self.list_value {
            Some(x) => x,
            None => panic!("Not list value"),
        }
    }

    pub fn get_map_value(&self) -> &HashMap<String, MgValue> {
        if self.value_type != MgValueType::Map {
            panic!("Not map value");
        }
        match &self.map_value {
            Some(x) => x,
            None => panic!("Not map value"),
        }
    }

    pub fn get_node_value(&self) -> &MgNode {
        if self.value_type != MgValueType::Node {
            panic!("Not node value");
        }
        match &self.node_value {
            Some(x) => x,
            None => panic!("Not node value"),
        }
    }

    pub fn get_relationship_value(&self) -> &MgRelationship {
        if self.value_type != MgValueType::Relationship {
            panic!("Not relationship value");
        }
        match &self.relationship_value {
            Some(x) => x,
            None => panic!("Not relationship value"),
        }
    }

    pub fn get_unbound_relationship_value(&self) -> &MgUnboundRelationship {
        if self.value_type != MgValueType::UnboundRelationship {
            panic!("Not unbound_relationship value");
        }
        match &self.unbound_relationship_value {
            Some(x) => x,
            None => panic!("Not unbound_relationship value"),
        }
    }

    pub fn get_path_value(&self) -> &MgPath {
        if self.value_type != MgValueType::Path {
            panic!("Not path value");
        }
        match &self.path_value {
            Some(x) => x,
            None => panic!("Not path value"),
        }
    }

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

impl fmt::Display for MgValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.value_type {
            MgValueType::Null => write!(f, "NULL"),
            MgValueType::Bool => write!(f, "{}", match self.bool_value {
                Some(x) => x.to_string(),
                None => String::from("NULL"),
            }),
            MgValueType::Int => write!(f, "{}", match self.int_value {
                Some(x) => x.to_string(),
                None => String::from("NULL"),
            }),
            MgValueType::Float => write!(f, "{}", match self.float_value {
                Some(x) => x.to_string(),
                None => String::from("NULL"),
            }),
            MgValueType::String => write!(f, "{}", match &self.string_value {
                Some(x) => format!("'{}'", x),
                None => String::from("NULL"),
            }),
            MgValueType::List => write!(f, "{}", match &self.list_value {
                Some(x) => x.iter().map(|val| val.to_string()).collect::<Vec<String>>().join(", "),
                None => String::from("NULL"),
            }),
            MgValueType::Map => write!(f, "{}", match &self.map_value {
                Some(x) => mg_map_to_string(x),
                None => String::from("NULL"),
            }),
            MgValueType::Node => write!(f, "{}", match &self.node_value {
                Some(x) => x.to_string(),
                None => String::from("NULL"),
            }),
            MgValueType::Relationship => write!(f, "{}", match &self.relationship_value {
                Some(x) => x.to_string(),
                None => String::from("NULL"),
            }),
            MgValueType::UnboundRelationship => write!(f, "{}", match &self.unbound_relationship_value {
                Some(x) => x.to_string(),
                None => String::from("NULL"),
            }),
            MgValueType::Path => write!(f, "{}", match &self.path_value {
                Some(x) => x.to_string(),
                None => String::from("NULL"),
            }),
            MgValueType::Unknown => write!(f, "NULL"),
        }
    }
}


fn mg_map_to_string(mg_map: &HashMap<String, MgValue>) -> String {
    let mut properties: Vec<String> = Vec::new();
    for (key, value) in mg_map {
        properties.push(format!("'{}': {}", key, value));
    }
    return format!("{{{}}}", properties.join(", "));
}

impl fmt::Display for MgNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(:{} {})", self.labels.join(", "), mg_map_to_string(&self.properties))
    }
}

impl fmt::Display for MgRelationship {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[:{} {}]", self.type_, mg_map_to_string(&self.properties))
    }
}

impl fmt::Display for MgUnboundRelationship {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[:{} {}]", self.type_, mg_map_to_string(&self.properties))
    }
}

// TODO: display for path
