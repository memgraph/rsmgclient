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

use rsmgclient::{connect, str_to_c_str, ConnectParams, MgValue};
use std::ffi::CStr;

extern "C" fn my_callback(
    hostname: *const ::std::os::raw::c_char,
    ip_address: *const ::std::os::raw::c_char,
    key_type: *const ::std::os::raw::c_char,
    fingerprint: *const ::std::os::raw::c_char,
    data: *mut ::std::os::raw::c_void,
) -> i32 {
    let host_str = unsafe { CStr::from_ptr(hostname).to_str().unwrap() };
    let ip_adr_str = unsafe { CStr::from_ptr(ip_address).to_str().unwrap() };
    let key_type_str = unsafe { CStr::from_ptr(key_type).to_str().unwrap() };
    let fingerprint_str = unsafe { CStr::from_ptr(fingerprint).to_str().unwrap() };
    let data_str = unsafe {
        CStr::from_ptr(data as *const ::std::os::raw::c_char)
            .to_str()
            .unwrap()
    };
    println!(
        "Hello from C\nhostname: {}\nip_adr: {}\nkey_type: {}\nfingerprint: {}\ntrust_data: {}",
        host_str, ip_adr_str, key_type_str, fingerprint_str, data_str
    );
    0
}

fn main() {
    let data = str_to_c_str("My trust data") as *mut ::std::os::raw::c_void;
    let connect_prms = ConnectParams {
        host: Some(String::from("localhost")),
        trust_data: Some(data),
        trust_callback: Some(my_callback),
        ..Default::default()
    };
    let connection = match connect(&connect_prms) {
        Ok(c) => c,
        Err(err) => panic!("{}", err),
    };
    // destroy data
    unsafe {
        Box::from_raw(data);
    };

    let rows: Vec<Vec<MgValue>> = match connection.execute(
        "CREATE (n:Person {name: 'John'})-[e:KNOWS]->(m:Person {name: 'Steve'}) RETURN n, e, m;",
    ) {
        Ok(res) => res,
        Err(err) => panic!("Query failed: {}", err),
    };

    for row in rows {
        for val in row {
            println!("{}", val);
        }
    }
}
