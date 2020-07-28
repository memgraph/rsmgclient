use super::*;
use std::ffi::{CStr, CString};

#[test]
fn from_c_mg_value_null() {
    let c_mg_value = unsafe { bindings::mg_value_make_null() };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValueType::Null, mg_value.value_type);
}

#[test]
fn from_c_mg_value_bool_false() {
    let c_mg_value = unsafe { bindings::mg_value_make_bool(0) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValueType::Bool, mg_value.value_type);
    assert_eq!(false, unsafe { mg_value.value.bool_value });
}

#[test]
fn from_c_mg_value_bool_true() {
    let c_mg_value = unsafe { bindings::mg_value_make_bool(27) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValueType::Bool, mg_value.value_type);
    assert_eq!(true, unsafe { mg_value.value.bool_value });
}

#[test]
fn from_c_mg_value_int() {
    let c_mg_value = unsafe { bindings::mg_value_make_integer(19) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValueType::Int, mg_value.value_type);
    assert_eq!(19, unsafe { mg_value.value.int_value });
}

#[test]
fn from_c_mg_value_float() {
    let c_mg_value = unsafe { bindings::mg_value_make_float(3.1465) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValueType::Float, mg_value.value_type);
    assert_eq!(3.1465, unsafe { mg_value.value.float_value });
}

#[test]
fn from_c_mg_value_string() {
    // TODO: add some complex symbols
    let c_str = CString::new(String::from("mg string!")).unwrap();
    let c_mg_value = unsafe { bindings::mg_value_make_string(c_str.as_ptr()) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValueType::String, mg_value.value_type);
    assert_eq!("mg string!", unsafe {
        (*mg_value.value.string_ptr).as_str()
    });
}

#[test]
fn from_c_mg_value_list() {
    // TODO: add at least one complex MgValue
    let c_mg_list = unsafe { bindings::mg_list_make_empty(3) };
    unsafe {
        bindings::mg_list_append(c_mg_list, bindings::mg_value_make_null());
        bindings::mg_list_append(c_mg_list, bindings::mg_value_make_bool(1));
        bindings::mg_list_append(c_mg_list, bindings::mg_value_make_integer(130));
    };
    let c_mg_value = unsafe { bindings::mg_value_make_list(c_mg_list) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValueType::List, mg_value.value_type);
    let mg_list = unsafe { &*mg_value.value.list_ptr };
    assert_eq!(3, mg_list.len());
    assert_eq!(MgValueType::Null, mg_list[0].value_type);
    assert_eq!(MgValueType::Bool, mg_list[1].value_type);
    assert_eq!(true, unsafe { mg_list[1].value.bool_value });
    assert_eq!(MgValueType::Int, mg_list[2].value_type);
    assert_eq!(130, unsafe { mg_list[2].value.int_value });
}

#[test]
fn from_c_mg_value_map() {
    // TODO: add at least 1 complex mg_value
    let c_mg_map = unsafe { bindings::mg_map_make_empty(3) };
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
    };
    let c_mg_value = unsafe { bindings::mg_value_make_map(c_mg_map) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    assert_eq!(MgValueType::Map, mg_value.value_type);
    let mg_map = unsafe { &*mg_value.value.map_ptr };
    assert_eq!(3, mg_map.len());
    assert_eq!(MgValueType::Null, mg_map.get("name").unwrap().value_type);
    assert_eq!(MgValueType::Bool, mg_map.get("is_it").unwrap().value_type);
    assert_eq!(true, unsafe {
        mg_map.get("is_it").unwrap().value.bool_value
    });
    assert_eq!(MgValueType::Int, mg_map.get("id").unwrap().value_type);
    assert_eq!(128, unsafe { mg_map.get("id").unwrap().value.int_value });
}

#[test]
fn from_c_mg_value_node() {
    let c_mg_map = unsafe { bindings::mg_map_make_empty(3) };
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
    };

    let c_mg_string = unsafe { bindings::mg_string_make(str_to_c_str("test")) };

    let mut values = Box::into_raw(Box::new(c_mg_string));

    let c_node = bindings::mg_node {
        id: 1,
        label_count: 1,
        labels: values,
        properties: c_mg_map,
    };

    let c_mg_value = unsafe { bindings::mg_value_make_node(bindings::mg_node_copy(&c_node)) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    let mg_node = mg_value_node(c_mg_value);
    let mg_node_ref = unsafe { &*mg_value.value.node_ptr };

    assert_eq!(MgValueType::Node, mg_value.value_type);
    assert_eq!(1, mg_node.labels.len());
    assert_eq!(3, mg_node.properties.len());
    assert_eq!(
        MgValueType::Null,
        mg_node_ref.properties.get("name").unwrap().value_type
    );
    assert_eq!(
        MgValueType::Bool,
        mg_node_ref.properties.get("is_it").unwrap().value_type
    );
    assert_eq!(true, unsafe {
        mg_node.properties.get("is_it").unwrap().value.bool_value
    });
    assert_eq!(
        MgValueType::Int,
        mg_node.properties.get("id").unwrap().value_type
    );
    assert_eq!(128, unsafe {
        mg_node.properties.get("id").unwrap().value.int_value
    });
    assert_eq!(1, mg_node.label_count);
    assert_eq!(1, unsafe { bindings::mg_node_id(&c_node) });
    assert_eq!(1, unsafe { bindings::mg_node_label_count(&c_node) });

    unsafe { bindings::mg_node_destroy(bindings::mg_node_copy(&c_node)) }
    unsafe { Box::from_raw(*values) };
}

#[test]
fn from_c_mg_value_relationship() {
    let c_mg_map = unsafe { bindings::mg_map_make_empty(3) };
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
    };

    let c_type = unsafe { bindings::mg_string_make(str_to_c_str("test")) };
    let c_relationship = bindings::mg_relationship{
        id: 1,
        start_id: 1,
        end_id: 2,
        type_: c_type,
        properties: c_mg_map,
    };

    let c_mg_value = unsafe { bindings::mg_value_make_relationship(bindings::mg_relationship_copy(&c_relationship)) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    let mg_relationship = mg_value_relationship(c_mg_value);
    let mg_relationship_ref = unsafe { &*mg_value.value.relationship_ptr };

    assert_eq!(MgValueType::Relationship, mg_value.value_type);
    assert_eq!(1, mg_relationship.start_id);
    assert_eq!(2, mg_relationship.end_id);
    assert_eq!(
        MgValueType::Null,
        mg_relationship_ref.properties.get("name").unwrap().value_type
    );
    assert_eq!(
        MgValueType::Bool,
        mg_relationship_ref.properties.get("is_it").unwrap().value_type
    );
    assert_eq!(true, unsafe {
        mg_relationship.properties.get("is_it").unwrap().value.bool_value
    });
    assert_eq!(
        MgValueType::Int,
        mg_relationship.properties.get("id").unwrap().value_type
    );
    assert_eq!(128, unsafe {
        mg_relationship.properties.get("id").unwrap().value.int_value
    });
    unsafe{
        assert_eq!(1,bindings::mg_relationship_id(&c_relationship) );
        assert_eq!(1,bindings::mg_relationship_start_id(&c_relationship) );
        assert_eq!(2,bindings::mg_relationship_end_id(&c_relationship) );

        let c_str = str_to_c_str("test");
        let c_mg_value = bindings::mg_value_make_string(c_str);
        assert_eq!(mg_string_to_string(bindings::mg_value_string(c_mg_value)),mg_string_to_string(bindings::mg_relationship_type(&c_relationship)));

        bindings::mg_relationship_destroy(bindings::mg_relationship_copy(&c_relationship));
    }
}

#[test]
fn from_c_mg_value_unbound_relationship() {
    let c_mg_map = unsafe { bindings::mg_map_make_empty(3) };
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
    };

    let c_type = unsafe { bindings::mg_string_make(str_to_c_str("test")) };
    let c_unbound_relationship = bindings::mg_unbound_relationship{
        id: 1,
        type_: c_type,
        properties: c_mg_map,
    };

    let c_mg_value = unsafe { bindings::mg_value_make_unbound_relationship(bindings::mg_unbound_relationship_copy(&c_unbound_relationship)) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    let mg_unbound_relationship = mg_value_unbound_relationship(c_mg_value);
    let mg_unbound_relationship_ref = unsafe { &*mg_value.value.unbound_relationship_ptr};

    assert_eq!(MgValueType::UnboundRelationship, mg_value.value_type);
    assert_eq!(
        MgValueType::Null,
        mg_unbound_relationship_ref.properties.get("name").unwrap().value_type
    );
    assert_eq!(
        MgValueType::Bool,
        mg_unbound_relationship_ref.properties.get("is_it").unwrap().value_type
    );
    assert_eq!(true, unsafe {
        mg_unbound_relationship.properties.get("is_it").unwrap().value.bool_value
    });
    assert_eq!(
        MgValueType::Int,
        mg_unbound_relationship.properties.get("id").unwrap().value_type
    );
    assert_eq!(128, unsafe {
        mg_unbound_relationship.properties.get("id").unwrap().value.int_value
    });
    unsafe{
        assert_eq!(1,bindings::mg_unbound_relationship_id(&c_unbound_relationship) );
        bindings::mg_unbound_relationship_destroy(bindings::mg_unbound_relationship_copy(&c_unbound_relationship));
    }

}

#[test]
fn from_c_mg_value_path() {

    let mut seq:i64=1;
    let seq_ptr:*mut i64=&mut seq;
    let c_mg_map = unsafe { bindings::mg_map_make_empty(3) };
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
    };

    let c_mg_string = unsafe { bindings::mg_string_make(str_to_c_str("test")) };
    let mut values = Box::into_raw(Box::new(c_mg_string));
    let mut c_node = bindings::mg_node {
        id: 1,
        label_count: 1,
        labels: values,
        properties: c_mg_map,
    };
    let c_node_ptr:*mut bindings::mg_node=&mut c_node;
    let mut nodes_box = Box::into_raw(Box::new(c_node_ptr));

    let c_type = unsafe { bindings::mg_string_make(str_to_c_str("test")) };
    let mut c_unbound_relationship = bindings::mg_unbound_relationship{
        id: 1,
        type_: c_type,
        properties: c_mg_map,
    };
    let c_unbound_relationship_ptr:*mut bindings::mg_unbound_relationship=&mut c_unbound_relationship;
    let mut unbound_relationship_box = Box::into_raw(Box::new(c_unbound_relationship_ptr));

    let c_path= bindings::mg_path {
        node_count: 1,
        relationship_count:1,
        sequence_length: 1,
        nodes: nodes_box,
        relationships: unbound_relationship_box,
        sequence: seq_ptr,
    };

    let c_mg_value = unsafe { bindings::mg_value_make_path(bindings::mg_path_copy(&c_path)) };
    let mg_value = unsafe { MgValue::from_mg_value(c_mg_value) };
    let mg_path = mg_value_path(c_mg_value);
    let mg_path_ref = unsafe { &*mg_value.value.path_ptr };

    /*let mg_path_node_at=unsafe{bindings::mg_path_node_at(&c_path, 1)};
    let c_mg_value1 = unsafe { bindings::mg_value_make_node(bindings::mg_node_copy(mg_path_node_at)) };
    let mg_value1 = unsafe { MgValue::from_mg_value(c_mg_value1) };
    let mg_node_ref1 = unsafe { &*mg_value1.value.node_ptr };*/

    unsafe{
    assert_eq!(MgValueType::Path, mg_value.value_type);
    assert_eq!(0,bindings::mg_path_length(&c_path));
    //assert_eq!(1, mg_node_ref1.id);
    }

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
