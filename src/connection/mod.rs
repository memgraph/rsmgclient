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
use super::error::MgError;
use super::mg_value::{
    c_string_to_string, hash_map_to_mg_map, mg_list_to_vec, MgValue, QueryParam,
};
use std::collections::HashMap;
use std::ffi::CString;

pub struct ConnectParams {
    pub port: u16,
    pub host: Option<String>,
    pub address: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub client_name: String,
    pub sslmode: SSLMode,
    pub sslcert: Option<String>,
    pub sslkey: Option<String>,
    pub trust_callback: Option<*const dyn Fn(&String, &String, &String, &String) -> i32>,
}

impl Default for ConnectParams {
    fn default() -> Self {
        ConnectParams {
            port: 7687,
            host: None,
            address: None,
            username: None,
            password: None,
            client_name: String::from("MemgraphBolt/0.1"),
            sslmode: SSLMode::Require,
            sslcert: None,
            sslkey: None,
            trust_callback: None,
        }
    }
}

#[derive(PartialEq)]
pub enum SSLMode {
    Disable,
    Require,
}

pub struct Connection {
    mg_session: *mut bindings::mg_session,
}

fn sslmode_to_c(sslmode: &SSLMode) -> u32 {
    match sslmode {
        SSLMode::Disable => bindings::mg_sslmode_MG_SSLMODE_DISABLE,
        SSLMode::Require => bindings::mg_sslmode_MG_SSLMODE_REQUIRE,
    }
}

fn read_error_message(mg_session: *mut bindings::mg_session) -> String {
    let c_error_message = unsafe { bindings::mg_session_error(mg_session) };
    unsafe { c_string_to_string(c_error_message) }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { bindings::mg_session_destroy(self.mg_session) };
    }
}

impl Connection {
    pub fn execute(
        &self,
        query: &str,
        params: Option<&HashMap<String, QueryParam>>,
    ) -> Result<Vec<Vec<MgValue>>, MgError> {
        let c_query = CString::new(query).unwrap();
        let mg_params = match params {
            Some(x) => hash_map_to_mg_map(x),
            None => std::ptr::null(),
        };
        let mut status = unsafe {
            bindings::mg_session_run(
                self.mg_session,
                c_query.as_ptr(),
                mg_params,
                &mut std::ptr::null(),
            )
        };

        if status != 0 {
            return Err(MgError::new(read_error_message(self.mg_session)));
        }

        let mut res: Vec<Vec<MgValue>> = Vec::new();
        unsafe {
            loop {
                let mut mg_result: *mut bindings::mg_result = std::ptr::null_mut();
                status = bindings::mg_session_pull(self.mg_session, &mut mg_result);
                let row = bindings::mg_result_row(mg_result);
                match status {
                    1 => res.push(mg_list_to_vec(row)),
                    // last row returned
                    0 => {}
                    _ => return Err(MgError::new(read_error_message(self.mg_session))),
                }

                if status != 1 {
                    break;
                }
            }
        };

        Ok(res)
    }
}

extern "C" fn trust_callback_wrapper(
    host: *const ::std::os::raw::c_char,
    ip_address: *const ::std::os::raw::c_char,
    key_type: *const ::std::os::raw::c_char,
    fingerprint: *const ::std::os::raw::c_char,
    fun_raw: *mut ::std::os::raw::c_void,
) -> ::std::os::raw::c_int {
    let fun: &mut &mut dyn Fn(&String, &String, &String, &String) -> i32 =
        unsafe { std::mem::transmute(fun_raw) };

    unsafe {
        fun(
            &c_string_to_string(host),
            &c_string_to_string(ip_address),
            &c_string_to_string(key_type),
            &c_string_to_string(fingerprint),
        ) as std::os::raw::c_int
    }
}

pub fn connect(param_struct: &ConnectParams) -> Result<Connection, MgError> {
    let mg_session_params = unsafe { bindings::mg_session_params_make() };
    let mut trust_callback_ptr = std::ptr::null_mut();
    unsafe {
        match &param_struct.host {
            Some(x) => bindings::mg_session_params_set_host(mg_session_params, str_to_c_str(x)),
            None => {}
        }
        bindings::mg_session_params_set_port(mg_session_params, param_struct.port);
        match &param_struct.address {
            Some(x) => bindings::mg_session_params_set_address(mg_session_params, str_to_c_str(x)),
            None => {}
        }
        match &param_struct.username {
            Some(x) => bindings::mg_session_params_set_username(mg_session_params, str_to_c_str(x)),
            None => {}
        }
        match &param_struct.password {
            Some(x) => bindings::mg_session_params_set_password(mg_session_params, str_to_c_str(x)),
            None => {}
        }
        bindings::mg_session_params_set_client_name(
            mg_session_params,
            str_to_c_str(&param_struct.client_name),
        );
        bindings::mg_session_params_set_sslmode(
            mg_session_params,
            sslmode_to_c(&param_struct.sslmode),
        );
        match &param_struct.sslcert {
            Some(x) => bindings::mg_session_params_set_sslcert(mg_session_params, str_to_c_str(x)),
            None => {}
        }
        match &param_struct.sslkey {
            Some(x) => bindings::mg_session_params_set_sslkey(mg_session_params, str_to_c_str(x)),
            None => {}
        }
        match &param_struct.trust_callback {
            Some(x) => {
                trust_callback_ptr = Box::into_raw(Box::new(*x));

                bindings::mg_session_params_set_trust_data(
                    mg_session_params,
                    trust_callback_ptr as *mut ::std::os::raw::c_void,
                );
                bindings::mg_session_params_set_trust_callback(
                    mg_session_params,
                    Some(trust_callback_wrapper),
                );
            }
            None => {}
        }
    }

    let mut mg_session: *mut bindings::mg_session = std::ptr::null_mut();
    let status = unsafe { bindings::mg_connect(mg_session_params, &mut mg_session) };
    unsafe {
        bindings::mg_session_params_destroy(mg_session_params);
        if !trust_callback_ptr.is_null() {
            Box::from_raw(trust_callback_ptr);
        }
    };

    if status != 0 {
        return Err(MgError::new(read_error_message(mg_session)));
    }

    Ok(Connection { mg_session })
}

// allocates memory and passes ownership, user is responsible for freeing object!
pub fn str_to_c_str(rust_str: &str) -> *const ::std::os::raw::c_char {
    let c_str_ptr: *mut CString = Box::into_raw(Box::from(CString::new(rust_str).unwrap()));
    unsafe { (*c_str_ptr).as_ptr() }
}
