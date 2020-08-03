use super::*;
use std::ffi::CString;

fn mg_value_to_c_mg_value(mg_value: &MgValue) -> *mut bindings::mg_value {
    unsafe {
        match mg_value {
            MgValue::Null => bindings::mg_value_make_null(),
            MgValue::Bool(x) => bindings::mg_value_make_bool(match x {
                false => 0,
                true => 1,
            }),
            MgValue::Int(x) => bindings::mg_value_make_integer(*x),
            MgValue::Float(x) => bindings::mg_value_make_float(*x),
            MgValue::String(x) => bindings::mg_value_make_string(str_to_c_str(x.as_str())),
            MgValue::List(x) => {
                bindings::mg_value_make_list(bindings::mg_list_copy(vector_to_mg_list(x)))
            }
            MgValue::Map(x) => {
                bindings::mg_value_make_map(bindings::mg_map_copy(hash_map_to_mg_map(x)))
            }
            MgValue::Node(x) => {
                let labels_box =
                    Box::into_raw(x.labels.into_boxed_slice()) as *mut *mut bindings::mg_string;
                let mut c_node = bindings::mg_node {
                    id: x.id,
                    label_count: x.label_count,
                    labels: labels_box,
                    properties: hash_map_to_mg_map(&x.properties),
                };
                bindings::mg_value_make_node(bindings::mg_node_copy(&c_node))
            }
            MgValue::Relationship(x) => {
                let c_type = unsafe { bindings::mg_string_make(str_to_c_str(&x.type_)) };
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
            MgValue::UnboundRelationship(x) => {
                let c_type = unsafe { bindings::mg_string_make(str_to_c_str(&x.type_)) };
                let mut c_unbound_relationship = bindings::mg_unbound_relationship {
                    id: x.id,
                    type_: c_type,
                    properties: hash_map_to_mg_map(&x.properties),
                };
                bindings::mg_value_make_unbound_relationship(
                    bindings::mg_unbound_relationship_copy(&c_unbound_relationship),
                )
            }
            MgValue::Path(x) => {
                let arr: i64 = 1;
                let boxed: Box<i64> = Box::new(arr);
                let seq_ptr: *mut i64 = Box::into_raw(boxed);
                let nodes_box =
                    Box::into_raw(x.nodes.into_boxed_slice()) as *mut *mut bindings::mg_node;
                let unbound_relationship_box = Box::into_raw(x.relationships.into_boxed_slice())
                    as *mut *mut bindings::mg_unbound_relationship;
                let c_path = bindings::mg_path {
                    node_count: x.node_count,
                    relationship_count: x.relationship_count,
                    sequence_length: x.node_count,
                    nodes: nodes_box,
                    relationships: unbound_relationship_box,
                    sequence: seq_ptr,
                };
                bindings::mg_value_make_path(bindings::mg_path_copy(&c_path))
            }
        }
    }
}
fn vector_to_mg_list(vector: &Vec<MgValue>) -> *mut bindings::mg_list {
    let size = vector.len() as u32;
    let mg_list = unsafe { bindings::mg_list_make_empty(size) };
    for mg_val in vector {
        unsafe {
            bindings::mg_list_append(mg_list, mg_value_to_c_mg_value(mg_val));
        };
    }
    mg_list
}

fn hash_map_to_mg_map(hash_map: &HashMap<String, MgValue>) -> *mut bindings::mg_map {
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
fn test() {
    let mg_map = hash_map_to_mg_map(&hashmap! {
        String::from("name") => MgValue::Null,
        String::from("is_it") => MgValue::Bool(true),
        String::from("id") => MgValue::Int(128),
        String::from("rel") => MgValue::Relationship(MgRelationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: String::from("test"),
            properties: hashmap!{
                String::from("name") => MgValue::Null,
            }
        }),
    });
}

#[test]
fn from_c_mg_value_null() {
    let c_mg_value = unsafe { bindings::mg_value_make_null() };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValue::Null, mg_value);
    assert_eq!(format!("{}", mg_value), "NULL");
}

#[test]
fn from_c_mg_value_bool_false() {
    let c_mg_value = unsafe { bindings::mg_value_make_bool(0) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValue::Bool(false), mg_value);
    assert_eq!(format!("{}", mg_value), "false");
}

#[test]
fn from_c_mg_value_bool_true() {
    let c_mg_value = unsafe { bindings::mg_value_make_bool(27) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValue::Bool(true), mg_value);
}

#[test]
fn from_c_mg_value_int() {
    let c_mg_value = unsafe { bindings::mg_value_make_integer(19) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValue::Int(19), mg_value);
    assert_eq!(format!("{}", mg_value), "19");
}

#[test]
fn from_c_mg_value_float() {
    let c_mg_value = unsafe { bindings::mg_value_make_float(3.1465) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValue::Float(3.1465), mg_value);
    assert_eq!(format!("{}", mg_value), "3.1465");
}

#[test]
fn from_c_mg_value_string() {
    let c_str = CString::new(String::from("ṰⱻⱾᵀ")).unwrap();
    let c_mg_value = unsafe { bindings::mg_value_make_string(c_str.as_ptr()) };

    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValue::String(String::from("ṰⱻⱾᵀ")), mg_value);
    assert_eq!(format!("{}", mg_value), "\'ṰⱻⱾᵀ\'");

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
fn from_c_mg_value_list() {
    let mg_values = vec![
        MgValue::Null,
        MgValue::Bool(true),
        MgValue::Int(130),
        MgValue::Relationship(MgRelationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: "test".to_string(),
            properties: hashmap! {
                String::from("name") => MgValue::Null,
            },
        }),
    ];

    let c_mg_value = unsafe { bindings::mg_value_make_list(vector_to_mg_list(&mg_values)) };
    let mut mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValue::List(mg_values), mg_value);

    mg_value = MgValue::List(vec![
        MgValue::Null,
        MgValue::Bool(true),
        MgValue::Int(130),
        MgValue::Relationship(MgRelationship {
            id: 1,
            start_id: 1,
            end_id: 2,
            type_: "test".to_string(),
            properties: hashmap! {
                String::from("name") => MgValue::Null,
            },
        }),
    ]);

    assert_eq!(
        format!("{}", mg_value),
        "NULL, true, 130, [:test {'name': NULL}]"
    );
}
/*
#[test]
fn from_c_mg_value_map_node_relationships_path() {
    let c_mg_map2 = unsafe { bindings::mg_map_make_empty(1) };
    unsafe {
        bindings::mg_map_insert(
            c_mg_map2,
            str_to_c_str("name"),
            bindings::mg_value_make_null(),
        );
    };
    let c_type = unsafe { bindings::mg_string_make(str_to_c_str("test")) };
    let c_relationship = bindings::mg_relationship {
        id: 1,
        start_id: 1,
        end_id: 2,
        type_: c_type,
        properties: c_mg_map2,
    };
    let c_mg_map = unsafe { bindings::mg_map_make_empty(4) };
    unsafe {
        bindings::mg_map_insert(
            c_mg_map,
            str_to_c_str("name"),
            bindings::mg_value_make_null(),
        );
        bindings::mg_map_insert(
            c_mg_map,
            str_to_c_str("is_it"),
            bindings::mg_value_make_bool(1),
        );
        bindings::mg_map_insert(
            c_mg_map,
            str_to_c_str("id"),
            bindings::mg_value_make_integer(128),
        );
        bindings::mg_map_insert(
            c_mg_map,
            str_to_c_str("rel"),
            bindings::mg_value_make_relationship(bindings::mg_relationship_copy(&c_relationship)),
        )
    };
    let c_mg_value = unsafe { bindings::mg_value_make_map(c_mg_map) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValueType::Map, mg_value.value_type);
    let mg_map = unsafe { &*mg_value.value.map_ptr };
    assert_eq!(4, mg_map.len());
    assert_eq!(MgValueType::Null, mg_map.get("name").unwrap().value_type);
    assert_eq!(MgValueType::Bool, mg_map.get("is_it").unwrap().value_type);
    assert_eq!(true, unsafe {
        mg_map.get("is_it").unwrap().value.bool_value
    });
    assert_eq!(MgValueType::Int, mg_map.get("id").unwrap().value_type);
    assert_eq!(128, unsafe { mg_map.get("id").unwrap().value.int_value });
    assert_eq!(
        MgValueType::Relationship,
        mg_map.get("rel").unwrap().value_type
    );
    assert_eq!(format!("{}",mg_value),"{\'id\': 128, \'is_it\': true, \'name\': NULL, \'rel\': [:test {\'name\': NULL}]}");
    let c_mg_value = unsafe { bindings::mg_value_make_map(c_mg_map) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };

    let get_map = MgValue::get_map_value(&mg_value);
    unsafe { assert_eq!((*mg_value.value.map_ptr).len(), get_map.len()) };
    assert_eq!(4, get_map.len());
    assert_eq!(MgValueType::Null, get_map.get("name").unwrap().value_type);
    assert_eq!(MgValueType::Bool, get_map.get("is_it").unwrap().value_type);
    assert_eq!(true, unsafe {
        get_map.get("is_it").unwrap().value.bool_value
    });
    assert_eq!(MgValueType::Int, get_map.get("id").unwrap().value_type);
    assert_eq!(128, unsafe { get_map.get("id").unwrap().value.int_value });
    assert_eq!(
        MgValueType::Relationship,
        get_map.get("rel").unwrap().value_type
    );

    let c_mg_string = unsafe { bindings::mg_string_make(str_to_c_str("test")) };
    let values = Box::into_raw(Box::new(c_mg_string));
    let mut c_node = bindings::mg_node {
        id: 1,
        label_count: 1,
        labels: values,
        properties: c_mg_map,
    };

    let c_mg_value = unsafe { bindings::mg_value_make_node(bindings::mg_node_copy(&c_node)) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    let mg_node = unsafe { &*mg_value.value.node_ptr };

    assert_eq!(MgValueType::Node, mg_value.value_type);
    assert_eq!(1, mg_node.labels.len());
    assert_eq!(4, mg_node.properties.len());
    assert_eq!(1, mg_node.id);
    assert_eq!(1, mg_node.label_count);
    assert_eq!(
        MgValueType::Null,
        mg_node.properties.get("name").unwrap().value_type
    );
    assert_eq!(format!("{}",&mg_value),"(:test {'id': 128, 'is_it': true, 'name': NULL, 'rel': [:test {'name': NULL}]})");

    let get_node = MgValue::get_node_value(&mg_value);
    unsafe { assert_eq!((*mg_value.value.node_ptr).id, get_node.id) };
    assert_eq!(1, get_node.labels.len());
    assert_eq!(4, get_node.properties.len());
    assert_eq!(1, get_node.id);
    assert_eq!(1, get_node.label_count);
    assert_eq!(
        MgValueType::Null,
        get_node.properties.get("name").unwrap().value_type
    );

    let c_type = unsafe { bindings::mg_string_make(str_to_c_str("test")) };
    let c_relationship2 = bindings::mg_relationship {
        id: 1,
        start_id: 1,
        end_id: 2,
        type_: c_type,
        properties: c_mg_map,
    };

    let c_mg_value = unsafe {
        bindings::mg_value_make_relationship(bindings::mg_relationship_copy(&c_relationship2))
    };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    let mg_relationship = unsafe { &*mg_value.value.relationship_ptr };

    assert_eq!(MgValueType::Relationship, mg_value.value_type);
    assert_eq!(1, mg_relationship.start_id);
    assert_eq!(2, mg_relationship.end_id);
    assert_eq!(
        MgValueType::Null,
        mg_relationship.properties.get("name").unwrap().value_type
    );
    assert_eq!(1, mg_relationship.id);
    assert_eq!("test", mg_relationship.type_);
    assert_eq!(4, mg_relationship.properties.len());
    assert_eq!(format!("{}", mg_value),"[:test {'id': 128, 'is_it': true, 'name': NULL, 'rel': [:test {'name': NULL}]}]");
    let c_mg_value = unsafe {
        bindings::mg_value_make_relationship(bindings::mg_relationship_copy(&c_relationship2))
    };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };

    let get_relationship = MgValue::get_relationship_value(&mg_value);
    unsafe { assert_eq!((*mg_value.value.relationship_ptr).id, get_relationship.id) };
    assert_eq!(1, get_relationship.start_id);
    assert_eq!(2, get_relationship.end_id);
    assert_eq!(
        MgValueType::Null,
        get_relationship.properties.get("name").unwrap().value_type
    );
    assert_eq!(1, get_relationship.id);
    assert_eq!("test", get_relationship.type_);
    assert_eq!(4, get_relationship.properties.len());

    let c_type = unsafe { bindings::mg_string_make(str_to_c_str("test")) };
    let mut c_unbound_relationship = bindings::mg_unbound_relationship {
        id: 1,
        type_: c_type,
        properties: c_mg_map,
    };

    let c_mg_value = unsafe {
        bindings::mg_value_make_unbound_relationship(bindings::mg_unbound_relationship_copy(
            &c_unbound_relationship,
        ))
    };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    let mg_unbound_relationship = unsafe { &*mg_value.value.unbound_relationship_ptr };

    assert_eq!(MgValueType::UnboundRelationship, mg_value.value_type);
    assert_eq!(
        MgValueType::Null,
        mg_unbound_relationship
            .properties
            .get("name")
            .unwrap()
            .value_type
    );
    assert_eq!(1, mg_unbound_relationship.id);
    assert_eq!("test", mg_unbound_relationship.type_);
    assert_eq!(4, mg_unbound_relationship.properties.len());
    assert_eq!(format!("{}", mg_value),"[:test {'id': 128, 'is_it': true, 'name': NULL, 'rel': [:test {'name': NULL}]}]");


    let get_unbound_relationship = MgValue::get_unbound_relationship_value(&mg_value);
    unsafe {
        assert_eq!(
            (*mg_value.value.unbound_relationship_ptr).id,
            get_unbound_relationship.id
        )
    };
    assert_eq!(
        MgValueType::Null,
        get_unbound_relationship
            .properties
            .get("name")
            .unwrap()
            .value_type
    );
    assert_eq!(1, get_unbound_relationship.id);
    assert_eq!("test", get_unbound_relationship.type_);
    assert_eq!(4, get_unbound_relationship.properties.len());

    let arr: i64 = 1;
    let boxed: Box<i64> = Box::new(arr);
    let seq_ptr: *mut i64 = Box::into_raw(boxed);
    let mut a = Box::new([std::ptr::null_mut(); 2]);
    a[0] = &mut c_node;
    let c_node2 = unsafe { bindings::mg_node_copy(&c_node) };
    a[1] = c_node2;
    let nodes_box = Box::into_raw(a) as *mut *mut bindings::mg_node;
    let c_unbound_relationship_ptr: *mut bindings::mg_unbound_relationship =
        &mut c_unbound_relationship;
    let unbound_relationship_box = Box::into_raw(Box::new(c_unbound_relationship_ptr));

    let c_path = bindings::mg_path {
        node_count: 2,
        relationship_count: 1,
        sequence_length: 2,
        nodes: nodes_box,
        relationships: unbound_relationship_box,
        sequence: seq_ptr,
    };

    let c_mg_value = unsafe { bindings::mg_value_make_path(bindings::mg_path_copy(&c_path)) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    let mg_path = unsafe { &*mg_value.value.path_ptr };

    assert_eq!(MgValueType::Path, mg_value.value_type);
    assert_eq!(2, mg_path.node_count);
    assert_eq!(1, mg_path.relationship_count);
    assert_eq!(2, mg_path.nodes.len());
    assert_eq!(1, mg_path.relationships.len());

    let get_path = MgValue::get_path_value(&mg_value);
    assert_eq!(2, get_path.node_count);
    assert_eq!(1, get_path.relationship_count);
    assert_eq!(2, get_path.nodes.len());
    assert_eq!(1, get_path.relationships.len());

    unsafe {
        Box::from_raw(values);
        Box::from_raw(nodes_box);
        Box::from_raw(unbound_relationship_box);
    };
}

#[test]
#[should_panic(expected = "Not map value")]
fn panic_from_c_mg_value_map(){
    let c_mg_value = unsafe { bindings::mg_value_make_bool(19) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    let get_map = MgValue::get_map_value(&mg_value);
    unsafe {
        assert_eq!((*mg_value.value.map_ptr).len(), get_map.len());
    };
}

#[test]
#[should_panic(expected = "Not node value")]
fn panic_from_c_mg_value_node(){
    let c_mg_value = unsafe { bindings::mg_value_make_bool(19) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    let get_node = MgValue::get_node_value(&mg_value);
    unsafe {
        assert_eq!((*mg_value.value.node_ptr).labels.len(), get_node.labels.len());
    };
}

#[test]
#[should_panic(expected = "Not relationship value")]
fn panic_from_c_mg_value_relationship(){
    let c_mg_value = unsafe { bindings::mg_value_make_bool(19) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    let get_relationship = MgValue::get_relationship_value(&mg_value);
    unsafe {
        assert_eq!((*mg_value.value.relationship_ptr).properties.len(), get_relationship.properties.len());
    };
}

#[test]
#[should_panic(expected = "Not unbound_relationship value")]
fn panic_from_c_mg_value_unbound_relationship(){
    let c_mg_value = unsafe { bindings::mg_value_make_bool(19) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    let get_unbound_relationship = MgValue::get_unbound_relationship_value(&mg_value);
    unsafe {
        assert_eq!((*mg_value.value.unbound_relationship_ptr).properties.len(), get_unbound_relationship.properties.len());
    };
}

#[test]
#[should_panic(expected = "Not path value")]
fn panic_from_c_mg_value_path(){
    let c_mg_value = unsafe { bindings::mg_value_make_bool(19) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    let get_path = MgValue::get_path_value(&mg_value);
    unsafe {
        assert_eq!((*mg_value.value.path_ptr).nodes.len(), get_path.nodes.len());
    };
}

#[test]
fn from_c_mg_value_unknown() {
    let c_mg_value = bindings::mg_value {
        type_: bindings::mg_value_type_MG_VALUE_TYPE_UNKNOWN,
        __bindgen_anon_1: bindings::mg_value__bindgen_ty_1 { bool_v: 0 },
    };
    let mg_value = unsafe { MgValue::from_mg_value(&c_mg_value as *const bindings::mg_value) };
    assert_eq!(MgValueType::Unknown, mg_value.value_type);
}

#[test]
fn from_to_c_mg_value() {
    let query_param_null = QueryParam::Null;
    let c_mg_value = unsafe { *(query_param_null.to_c_mg_value()) };
    assert_eq!(c_mg_value.type_, bindings::mg_value_type_MG_VALUE_TYPE_NULL);

    let query_param_false = QueryParam::Bool(false);
    let c_mg_value = unsafe { *(query_param_false.to_c_mg_value()) };
    assert_eq!(c_mg_value.type_, bindings::mg_value_type_MG_VALUE_TYPE_BOOL);

    let query_param_true = QueryParam::Bool(true);
    let c_mg_value = unsafe { *(query_param_true.to_c_mg_value()) };
    assert_eq!(c_mg_value.type_, bindings::mg_value_type_MG_VALUE_TYPE_BOOL);

    let query_param_int = QueryParam::Int(20);
    let c_mg_value = unsafe { *(query_param_int.to_c_mg_value()) };
    let mg_value = unsafe { MgValue::from_mg_value(&c_mg_value) };
    assert_eq!(
        c_mg_value.type_,
        bindings::mg_value_type_MG_VALUE_TYPE_INTEGER
    );
    assert_eq!(unsafe { mg_value.value.int_value }, 20);

    let query_param_float = QueryParam::Float(3.15);
    let c_mg_value = unsafe { *(query_param_float.to_c_mg_value()) };
    let mg_value = unsafe { MgValue::from_mg_value(&c_mg_value) };
    assert_eq!(
        c_mg_value.type_,
        bindings::mg_value_type_MG_VALUE_TYPE_FLOAT
    );
    assert_eq!(unsafe { mg_value.value.float_value }, 3.15);

    let mut vec: Vec<QueryParam> = Vec::new();
    vec.push(query_param_null);
    vec.push(query_param_true);
    vec.push(query_param_int);
    vec.push(query_param_float);
    let query_param_list = QueryParam::List(vec);
    let c_mg_value = unsafe { *(query_param_list.to_c_mg_value()) };
    let mg_value = unsafe { MgValue::from_mg_value(&c_mg_value) };
    let mg_list = unsafe { &*mg_value.value.list_ptr };
    assert_eq!(c_mg_value.type_, bindings::mg_value_type_MG_VALUE_TYPE_LIST);
    assert_eq!(4, mg_list.len());
    assert_eq!(MgValueType::Null, mg_list[0].value_type);
    assert_eq!(MgValueType::Bool, mg_list[1].value_type);
    assert_eq!(true, unsafe { mg_list[1].value.bool_value });
    assert_eq!(MgValueType::Int, mg_list[2].value_type);
    assert_eq!(20, unsafe { mg_list[2].value.int_value });
    assert_eq!(MgValueType::Float, mg_list[3].value_type);
    assert_eq!(3.15, unsafe { mg_list[3].value.float_value });

    let mut map: HashMap<String, QueryParam> = HashMap::new();
    map.insert("null".to_string(), QueryParam::Null);
    map.insert("true".to_string(), QueryParam::Bool(true));
    map.insert("int".to_string(), QueryParam::Int(20));
    map.insert("float".to_string(), QueryParam::Float(3.15));
    let query_param_map = QueryParam::Map(map);
    let c_mg_value = unsafe { *(query_param_map.to_c_mg_value()) };
    let mg_value = unsafe { MgValue::from_mg_value(&c_mg_value) };
    let mg_map = unsafe { &*mg_value.value.map_ptr };
    assert_eq!(c_mg_value.type_, bindings::mg_value_type_MG_VALUE_TYPE_MAP);
    assert_eq!(4, mg_map.len());
    assert_eq!(MgValueType::Null, mg_map.get("null").unwrap().value_type);
    assert_eq!(MgValueType::Bool, mg_map.get("true").unwrap().value_type);
    assert_eq!(true, unsafe {
        mg_map.get("true").unwrap().value.bool_value
    });
    assert_eq!(MgValueType::Int, mg_map.get("int").unwrap().value_type);
    assert_eq!(20, unsafe { mg_map.get("int").unwrap().value.int_value });
    assert_eq!(MgValueType::Float, mg_map.get("float").unwrap().value_type);
    assert_eq!(3.15, unsafe {
        mg_map.get("float").unwrap().value.float_value
    });
}
*/
