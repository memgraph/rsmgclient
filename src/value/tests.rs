use super::*;
use std::ffi::CString;
use std::mem;
extern crate libc;

unsafe fn to_c_pointer_array<T, S>(
    vec: &[T],
    convert_fun: impl Fn(&T) -> *mut S,
) -> *mut *mut S {
    let size = vec.len() * mem::size_of::<*mut std::os::raw::c_void>();
    let ptr = libc::malloc(size) as *mut *mut S;
    for (i, el) in vec.iter().enumerate() {
        *ptr.add(i) = convert_fun(&el);
    }
    ptr
}

unsafe fn to_array_of_strings(vec: &[String]) -> *mut *mut bindings::mg_string {
    to_c_pointer_array(vec, |el| {
        bindings::mg_string_make(str_to_c_str(el.as_str()))
    })
}

unsafe fn to_array_of_nodes(vec: &[Node]) -> *mut *mut bindings::mg_node {
    to_c_pointer_array(vec, |el| {
        bindings::mg_node_copy(&bindings::mg_node {
            id: el.id,
            label_count: el.label_count,
            labels: to_array_of_strings(&el.labels),
            properties: hash_map_to_mg_map(&el.properties),
        })
    })
}

unsafe fn to_array_of_unbound_relationships(
    vec: &[UnboundRelationship],
) -> *mut *mut bindings::mg_unbound_relationship {
    to_c_pointer_array(vec, |x| {
        bindings::mg_unbound_relationship_copy(&bindings::mg_unbound_relationship {
            id: x.id,
            type_: bindings::mg_string_make(str_to_c_str(x.type_.as_str())),
            properties: hash_map_to_mg_map(&x.properties),
        })
    })
}

unsafe fn to_c_int_array(vec: &[i64]) -> *mut i64 {
    let size = vec.len() * mem::size_of::<i64>();
    let ptr = libc::malloc(size) as *mut i64;
    for (i, el) in vec.iter().enumerate() {
        *ptr.add(i) = *el;
    }
    ptr
}

fn mg_value_to_c_mg_value(mg_value: &Value) -> *mut bindings::mg_value {
    unsafe {
        match mg_value {
            Value::Null => bindings::mg_value_make_null(),
            Value::Bool(x) => bindings::mg_value_make_bool(match x {
                false => 0,
                true => 1,
            }),
            Value::Int(x) => bindings::mg_value_make_integer(*x),
            Value::Float(x) => bindings::mg_value_make_float(*x),
            Value::String(x) => bindings::mg_value_make_string(str_to_c_str(x.as_str())),
            Value::List(x) => {
                bindings::mg_value_make_list(bindings::mg_list_copy(vector_to_mg_list(x)))
            }
            Value::Map(x) => {
                bindings::mg_value_make_map(bindings::mg_map_copy(hash_map_to_mg_map(x)))
            }
            Value::Node(x) => {
                let labels_box = to_array_of_strings(&x.labels);
                let c_node = bindings::mg_node {
                    id: x.id,
                    label_count: x.label_count,
                    labels: labels_box,
                    properties: hash_map_to_mg_map(&x.properties),
                };
                bindings::mg_value_make_node(bindings::mg_node_copy(&c_node))
            }
            Value::Relationship(x) => {
                let c_type = bindings::mg_string_make(str_to_c_str(&x.type_));
                let c_relationship2 = bindings::mg_relationship {
                    id: x.id,
                    start_id: x.start_id,
                    end_id: x.end_id,
                    type_: c_type,
                    properties: hash_map_to_mg_map(&x.properties),
                };
                bindings::mg_value_make_relationship(bindings::mg_relationship_copy(
                    &c_relationship2,
                ))
            }
            Value::UnboundRelationship(x) => {
                let c_type = bindings::mg_string_make(str_to_c_str(&x.type_));
                let c_unbound_relationship = bindings::mg_unbound_relationship {
                    id: x.id,
                    type_: c_type,
                    properties: hash_map_to_mg_map(&x.properties),
                };
                bindings::mg_value_make_unbound_relationship(
                    bindings::mg_unbound_relationship_copy(&c_unbound_relationship),
                )
            }
            Value::Path(x) => {
                let mut sequence = Vec::new();
                for i in 0..x.node_count {
                    sequence.push(x.nodes[i as usize].id);
                    if i < x.relationship_count {
                        sequence.push(x.relationships[i as usize].id);
                    }
                }
                let sequence_length = sequence.len() as u32;
                let seq_ptr = to_c_int_array(&sequence);

                let nodes_box = to_array_of_nodes(&x.nodes);
                let unbound_relationship_box = to_array_of_unbound_relationships(&x.relationships);
                let c_path = bindings::mg_path {
                    node_count: x.node_count,
                    relationship_count: x.relationship_count,
                    sequence_length,
                    nodes: nodes_box,
                    relationships: unbound_relationship_box,
                    sequence: seq_ptr,
                };
                bindings::mg_value_make_path(bindings::mg_path_copy(&c_path))
            }
        }
    }
}
fn vector_to_mg_list(vector: &[Value]) -> *mut bindings::mg_list {
    let size = vector.len() as u32;
    let mg_list = unsafe { bindings::mg_list_make_empty(size) };
    for mg_val in vector {
        unsafe {
            bindings::mg_list_append(mg_list, mg_value_to_c_mg_value(mg_val));
        };
    }
    mg_list
}

fn hash_map_to_mg_map(hash_map: &HashMap<String, Value>) -> *mut bindings::mg_map {
    let size = hash_map.len();
    let mg_map = unsafe { bindings::mg_map_make_empty(size as u32) };
    for (key, val) in hash_map {
        unsafe {
            bindings::mg_map_insert(mg_map, str_to_c_str(key), mg_value_to_c_mg_value(val));
        }
    }

    mg_map
}

#[test]
fn from_c_mg_value_null() {
    let c_mg_value = unsafe { bindings::mg_value_make_null() };
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(Value::Null, mg_value);
    assert_eq!(format!("{}", mg_value), "NULL");
}

#[test]
fn from_c_mg_value_null_display() {
    let c_mg_value = unsafe { bindings::mg_value_make_null() };
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(format!("{}", mg_value), "NULL");
}

#[test]
fn from_c_mg_value_bool_false() {
    let c_mg_value = unsafe { bindings::mg_value_make_bool(0) };
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(Value::Bool(false), mg_value);
    assert_eq!(format!("{}", mg_value), "false");
}

#[test]
fn from_c_mg_value_bool_false_display() {
    let c_mg_value = unsafe { bindings::mg_value_make_bool(0) };
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(format!("{}", mg_value), "false");
}

#[test]
fn from_c_mg_value_bool_true() {
    let c_mg_value = unsafe { bindings::mg_value_make_bool(27) };
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(Value::Bool(true), mg_value);
}

#[test]
fn from_c_mg_value_int() {
    let c_mg_value = unsafe { bindings::mg_value_make_integer(19) };
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(Value::Int(19), mg_value);
    assert_eq!(format!("{}", mg_value), "19");
}

#[test]
fn from_c_mg_value_int_display() {
    let c_mg_value = unsafe { bindings::mg_value_make_integer(19) };
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(format!("{}", mg_value), "19");
}

#[test]
fn from_c_mg_value_float() {
    let c_mg_value = unsafe { bindings::mg_value_make_float(3.1465) };
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(Value::Float(3.1465), mg_value);
    assert_eq!(format!("{}", mg_value), "3.1465");
}

#[test]
fn from_c_mg_value_float_display() {
    let c_mg_value = unsafe { bindings::mg_value_make_float(3.1465) };
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(format!("{}", mg_value), "3.1465");
}

#[test]
fn from_c_mg_value_string() {
    let c_str = CString::new(String::from("ṰⱻⱾᵀ")).unwrap();
    let c_mg_value = unsafe { bindings::mg_value_make_string(c_str.as_ptr()) };

    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(Value::String(String::from("ṰⱻⱾᵀ")), mg_value);
    assert_eq!(format!("{}", mg_value), "\'ṰⱻⱾᵀ\'");
}

#[test]
fn from_c_mg_value_list() {
    let mg_values = vec![
        Value::Null,
        Value::Bool(true),
        Value::Int(130),
        Value::Relationship(Relationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: "test".to_string(),
            properties: hashmap! {
                String::from("name") => Value::Null,
            },
        }),
    ];

    let c_mg_value = unsafe { bindings::mg_value_make_list(vector_to_mg_list(&mg_values)) };
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(Value::List(mg_values), mg_value);
}

#[test]
fn from_c_mg_value_list_display() {
    let mg_value = Value::List(vec![
        Value::Null,
        Value::Bool(true),
        Value::Int(130),
        Value::Relationship(Relationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: "test".to_string(),
            properties: hashmap! {
                String::from("name") => Value::Null,
            },
        }),
    ]);

    assert_eq!(
        format!("{}", mg_value),
        "NULL, true, 130, [:test {'name': NULL}]"
    );
}

#[test]
fn from_c_mg_value_map() {
    let mg_map = hashmap! {
        String::from("name") => Value::Null,
        String::from("is_it") => Value::Bool(true),
        String::from("id") => Value::Int(128),
        String::from("rel") => Value::Relationship(Relationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: String::from("test"),
            properties: hashmap!{
                String::from("name") => Value::Null,
            }
        }),
    };

    let c_mg_value = unsafe { bindings::mg_value_make_map(hash_map_to_mg_map(&mg_map)) };
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(Value::Map(mg_map), mg_value);

    assert_eq!(
        format!("{}", mg_value),
        "{'id': 128, 'is_it': true, 'name': NULL, 'rel': [:test {'name': NULL}]}"
    );
}

#[test]
fn from_c_mg_value_map_display() {
    let mg_map = Value::Map(hashmap! {
        String::from("name") => Value::Null,
        String::from("is_it") => Value::Bool(true),
        String::from("id") => Value::Int(128),
        String::from("rel") => Value::Relationship(Relationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: String::from("test"),
            properties: hashmap!{
                String::from("name") => Value::Null,
            }
        }),
    });

    assert_eq!(
        format!("{}", mg_map),
        "{'id': 128, 'is_it': true, 'name': NULL, 'rel': [:test {'name': NULL}]}"
    );
}

#[test]
fn from_c_mg_value_node() {
    let values = vec![String::from("test")];
    let mg_map = hashmap! {
        String::from("id") => Value::Int(128),
        String::from("rel") => Value::Relationship(Relationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: String::from("test"),
            properties: hashmap!{
                String::from("name") => Value::Null,
            }
        }),
    };
    let c_node = Value::Node(Node {
        id: 1,
        label_count: 1,
        labels: values,
        properties: mg_map,
    });

    let c_mg_value = mg_value_to_c_mg_value(&c_node);
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(c_node, mg_value);

    //assert_eq!(format!("{}",value),"{'id': 128, 'is_it': true, 'name': NULL, 'rel': [:test {'name': NULL}]}");
}

#[test]
fn from_c_mg_value_node_display() {
    let values = vec![String::from("test")];
    let mg_map = hashmap! {
        String::from("id") => Value::Int(128),
        String::from("rel") => Value::Relationship(Relationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: String::from("test"),
            properties: hashmap!{
                String::from("name") => Value::Null,
            }
        }),
    };
    let c_node = Value::Node(Node {
        id: 1,
        label_count: 1,
        labels: values,
        properties: mg_map,
    });

    assert_eq!(
        format!("{}", c_node),
        "(:test {'id': 128, 'rel': [:test {'name': NULL}]})"
    );
}

#[test]
fn from_c_mg_value_relationship() {
    let mg_map = hashmap! {
        String::from("id") => Value::Int(128),
        String::from("rel") => Value::Relationship(Relationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: String::from("test"),
            properties: hashmap!{
                String::from("name") => Value::Null,
            }
        }),
    };
    let c_relationship = Value::Relationship(Relationship {
        id: 1,
        start_id: 1,
        end_id: 2,
        type_: String::from("test"),
        properties: mg_map,
    });

    let c_mg_value = mg_value_to_c_mg_value(&c_relationship);
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(c_relationship, mg_value);
}

#[test]
fn from_c_mg_value_relationship_display() {
    let mg_map = hashmap! {
        String::from("id") => Value::Int(128),
        String::from("rel") => Value::Relationship(Relationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: String::from("test"),
            properties: hashmap!{
                String::from("name") => Value::Null,
            }
        }),
    };
    let c_relationship = Value::Relationship(Relationship {
        id: 1,
        start_id: 1,
        end_id: 2,
        type_: String::from("test"),
        properties: mg_map,
    });

    assert_eq!(
        format!("{}", c_relationship),
        "[:test {'id': 128, 'rel': [:test {'name': NULL}]}]"
    );
}

#[test]
fn from_c_mg_value_unbound_relationship() {
    let mg_map = hashmap! {
        String::from("id") => Value::Int(128),
        String::from("rel") => Value::Relationship(Relationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: String::from("test"),
            properties: hashmap!{
                String::from("name") => Value::Null,
            }
        }),
    };
    let c_unbound_relationship = Value::UnboundRelationship(UnboundRelationship {
        id: 1,
        type_: String::from("test"),
        properties: mg_map,
    });

    let c_mg_value = mg_value_to_c_mg_value(&c_unbound_relationship);
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(c_unbound_relationship, mg_value);
}

#[test]
fn from_c_mg_value_unbound_relationship_display() {
    let mg_map = hashmap! {
        String::from("id") => Value::Int(128),
        String::from("rel") => Value::Relationship(Relationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: String::from("test"),
            properties: hashmap!{
                String::from("name") => Value::Null,
            }
        }),
    };
    let c_unbound_relationship = Value::UnboundRelationship(UnboundRelationship {
        id: 1,
        type_: String::from("test"),
        properties: mg_map,
    });
    assert_eq!(
        format!("{}", c_unbound_relationship),
        "[:test {'id': 128, 'rel': [:test {'name': NULL}]}]"
    );
}

#[test]
fn from_c_mg_value_path() {
    let values = vec![String::from("test")];
    let values2 = vec![String::from("test")];
    let mg_map = hashmap! {
        String::from("id") => Value::Int(128),
        String::from("rel") => Value::Relationship(Relationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: String::from("test"),
            properties: hashmap!{
                String::from("name") => Value::Null,
            }
        }),
    };
    let c_unbound_relationship = UnboundRelationship {
        id: 1,
        type_: String::from("test"),
        properties: mg_map,
    };
    let mg_map2 = hashmap! {
        String::from("id") => Value::Int(128),
        String::from("rel") => Value::Relationship(Relationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: String::from("test"),
            properties: hashmap!{
                String::from("name") => Value::Null,
            }
        }),
    };
    let mg_map3 = hashmap! {
        String::from("id") => Value::Int(128),
        String::from("rel") => Value::Relationship(Relationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: String::from("test"),
            properties: hashmap!{
                String::from("name") => Value::Null,
            }
        }),
    };
    let c_node = Node {
        id: 1,
        label_count: 1,
        labels: values,
        properties: mg_map2,
    };
    let c_node2 = Node {
        id: 1,
        label_count: 1,
        labels: values2,
        properties: mg_map3,
    };
    let c_path = Value::Path(Path {
        node_count: 2,
        relationship_count: 1,
        nodes: vec![c_node, c_node2],
        relationships: vec![c_unbound_relationship],
    });

    let c_mg_value = mg_value_to_c_mg_value(&c_path);
    let mg_value = unsafe { Value::from_mg_value(c_mg_value) };
    assert_eq!(c_path, mg_value);
}

#[test]
fn from_to_c_mg_value_null() {
    let query_param_null = QueryParam::Null;
    let c_mg_value = unsafe { *(query_param_null.to_c_mg_value()) };
    assert_eq!(c_mg_value.type_, bindings::mg_value_type_MG_VALUE_TYPE_NULL);
}

#[test]
fn from_to_c_mg_value_false() {
    let query_param_false = QueryParam::Bool(false);
    let c_mg_value = unsafe { *(query_param_false.to_c_mg_value()) };
    assert_eq!(c_mg_value.type_, bindings::mg_value_type_MG_VALUE_TYPE_BOOL);
}

#[test]
fn from_to_c_mg_value_true() {
    let query_param_true = QueryParam::Bool(true);
    let c_mg_value = unsafe { *(query_param_true.to_c_mg_value()) };
    assert_eq!(c_mg_value.type_, bindings::mg_value_type_MG_VALUE_TYPE_BOOL);
}

#[test]
fn from_to_c_mg_value_int() {
    let query_param_int = QueryParam::Int(20);
    let c_mg_value = unsafe { *(query_param_int.to_c_mg_value()) };
    let mg_value = unsafe { Value::from_mg_value(&c_mg_value) };
    assert_eq!(
        c_mg_value.type_,
        bindings::mg_value_type_MG_VALUE_TYPE_INTEGER
    );
    assert_eq!(mg_value.to_string(), "20");
}

#[test]
fn from_to_c_mg_value_float() {
    let query_param_float = QueryParam::Float(3.15);
    let c_mg_value = unsafe { *(query_param_float.to_c_mg_value()) };
    let mg_value = unsafe { Value::from_mg_value(&c_mg_value) };
    assert_eq!(
        c_mg_value.type_,
        bindings::mg_value_type_MG_VALUE_TYPE_FLOAT
    );
    assert_eq!(mg_value.to_string(), "3.15");
}

#[test]
fn from_to_c_mg_value_string() {
    let query_param = QueryParam::String("test".to_string());
    let c_mg_value = unsafe { *(query_param.to_c_mg_value()) };
    assert_eq!(
        c_mg_value.type_,
        bindings::mg_value_type_MG_VALUE_TYPE_STRING
    );
    assert_eq!(
        unsafe { mg_string_to_string(c_mg_value.__bindgen_anon_1.string_v) },
        String::from("test")
    );
}

#[test]
fn from_to_c_mg_value_list() {
    let mut vec: Vec<QueryParam> = Vec::new();
    vec.push(QueryParam::Null);
    vec.push(QueryParam::Bool(true));
    vec.push(QueryParam::Int(20));
    vec.push(QueryParam::Float(3.15));
    let query_param_list = QueryParam::List(vec);
    let c_mg_value = unsafe { *(query_param_list.to_c_mg_value()) };
    let mg_value = unsafe { Value::from_mg_value(&c_mg_value) };
    assert_eq!(c_mg_value.type_, bindings::mg_value_type_MG_VALUE_TYPE_LIST);
    let _c_mg_list = mg_value_to_c_mg_value(&mg_value);
}

#[test]
fn from_to_c_mg_value_map() {
    let mut map: HashMap<String, QueryParam> = HashMap::new();
    map.insert("null".to_string(), QueryParam::Null);
    map.insert("true".to_string(), QueryParam::Bool(true));
    map.insert("int".to_string(), QueryParam::Int(20));
    map.insert("float".to_string(), QueryParam::Float(3.15));
    let query_param_map = QueryParam::Map(map);
    let c_mg_value = unsafe { *(query_param_map.to_c_mg_value()) };
    let _mg_value = unsafe { Value::from_mg_value(&c_mg_value) };

    assert_eq!(c_mg_value.type_, bindings::mg_value_type_MG_VALUE_TYPE_MAP);
}

#[test]
fn from_c_mg_value_unknown() {
    let c_mg_value = bindings::mg_value {
        type_: bindings::mg_value_type_MG_VALUE_TYPE_UNKNOWN,
        __bindgen_anon_1: bindings::mg_value__bindgen_ty_1 { bool_v: 0 },
    };
    let _mg_value = unsafe { Value::from_mg_value(&c_mg_value as *const bindings::mg_value) };
    unsafe {
        assert_eq!(
            c_mg_value.type_,
            bindings::mg_value_get_type(&c_mg_value as *const bindings::mg_value)
        )
    };
}
