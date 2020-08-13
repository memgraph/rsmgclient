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
use super::value::{
    c_string_to_string, hash_map_to_mg_map, mg_list_to_vec, mg_value_string, str_to_c_str,
    QueryParam, Record,
};
use std::collections::HashMap;
use std::ffi::CString;
use std::vec::IntoIter;

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
    pub lazy: bool,
    pub autocommit: bool,
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
            lazy: true,
            autocommit: false,
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
    lazy: bool,
    autocommit: bool,
    in_transaction: bool,
    status: ConnectionStatus,
    results_iter: Option<IntoIter<Record>>,
    arraysize: u32,
}

#[derive(PartialEq)]
pub enum ConnectionStatus {
    Ready,
    Executing,
    Closed,
    Bad,
}

fn sslmode_to_c(sslmode: &SSLMode) -> u32 {
    match sslmode {
        SSLMode::Disable => bindings::mg_sslmode_MG_SSLMODE_DISABLE,
        SSLMode::Require => bindings::mg_sslmode_MG_SSLMODE_REQUIRE,
    }
}

fn read_error_message(mg_session: *mut bindings::mg_session) -> String {
    let c_error_message = unsafe { bindings::mg_session_error(mg_session) };
    unsafe { c_string_to_string(c_error_message, None) }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { bindings::mg_session_destroy(self.mg_session) };
    }
}

impl Connection {
    pub fn get_lazy(&self) -> bool {
        self.lazy
    }

    pub fn get_autocommit(&self) -> bool {
        self.autocommit
    }

    pub fn get_arraysize(&self) -> u32 {
        self.arraysize
    }

    pub fn get_in_transaction(&self) -> bool {
        self.in_transaction
    }

    pub fn get_status(&self) -> &ConnectionStatus {
        &self.status
    }

    pub fn set_lazy(&mut self, lazy: bool) {
        match self.status {
            ConnectionStatus::Ready => self.lazy = lazy,
            ConnectionStatus::Executing => panic!("Can't set lazy while executing"),
            ConnectionStatus::Bad => panic!("Bad connection"),
            ConnectionStatus::Closed => panic!("Connection is closed"),
        }
    }

    pub fn set_autocommit(&mut self, autocommit: bool) {
        if self.in_transaction {
            panic!("Can't set autocommit while in pending transaction");
        }

        match self.status {
            ConnectionStatus::Ready => self.autocommit = autocommit,
            ConnectionStatus::Executing => panic!("Can't set autocommit while executing"),
            ConnectionStatus::Bad => panic!("Bad connection"),
            ConnectionStatus::Closed => panic!("Connection is closed"),
        }
    }

    pub fn set_arraysize(&mut self, arraysize: u32) {
        self.arraysize = arraysize;
    }

    pub fn connect(param_struct: &ConnectParams) -> Result<Connection, MgError> {
        let mg_session_params = unsafe { bindings::mg_session_params_make() };
        unsafe {
            match &param_struct.host {
                Some(x) => bindings::mg_session_params_set_host(mg_session_params, str_to_c_str(x)),
                None => {}
            }
            bindings::mg_session_params_set_port(mg_session_params, param_struct.port);
            match &param_struct.address {
                Some(x) => {
                    bindings::mg_session_params_set_address(mg_session_params, str_to_c_str(x))
                }
                None => {}
            }
            match &param_struct.username {
                Some(x) => {
                    bindings::mg_session_params_set_username(mg_session_params, str_to_c_str(x))
                }
                None => {}
            }
            match &param_struct.password {
                Some(x) => {
                    bindings::mg_session_params_set_password(mg_session_params, str_to_c_str(x))
                }
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
                Some(x) => {
                    bindings::mg_session_params_set_sslcert(mg_session_params, str_to_c_str(x))
                }
                None => {}
            }
            match &param_struct.sslkey {
                Some(x) => {
                    bindings::mg_session_params_set_sslkey(mg_session_params, str_to_c_str(x))
                }
                None => {}
            }
            match &param_struct.trust_callback {
                Some(x) => {
                    let trust_callback_ptr = Box::into_raw(Box::new(*x));

                    bindings::mg_session_params_set_trust_data(
                        mg_session_params,
                        trust_callback_ptr as *mut ::std::os::raw::c_void,
                    );
                    bindings::mg_session_params_set_trust_callback(
                        mg_session_params,
                        Some(trust_callback_wrapper),
                    );

                    Box::from_raw(trust_callback_ptr);
                }
                None => {}
            }
        }

        let mut mg_session: *mut bindings::mg_session = std::ptr::null_mut();
        let status = unsafe { bindings::mg_connect(mg_session_params, &mut mg_session) };
        unsafe {
            bindings::mg_session_params_destroy(mg_session_params);
        };

        if status != 0 {
            return Err(MgError::new(read_error_message(mg_session)));
        }

        Ok(Connection {
            mg_session,
            lazy: param_struct.lazy,
            autocommit: param_struct.autocommit,
            in_transaction: false,
            status: ConnectionStatus::Ready,
            results_iter: None,
            arraysize: 1,
        })
    }

    fn connection_run_without_results(&mut self, query: &str) -> Result<(), MgError> {
        match unsafe {
            bindings::mg_session_run(
                self.mg_session,
                str_to_c_str(query),
                std::ptr::null(),
                std::ptr::null_mut(),
            )
        } {
            0 => {}
            _ => {
                self.status = ConnectionStatus::Bad;
                return Err(MgError::new(read_error_message(self.mg_session)));
            }
        }

        let mut result = std::ptr::null_mut();
        match unsafe { bindings::mg_session_pull(self.mg_session, &mut result) } {
            0 => Ok(()),
            _ => Err(MgError::new(read_error_message(self.mg_session))),
        }
    }

    pub fn execute(
        &mut self,
        query: &str,
        params: Option<&HashMap<String, QueryParam>>,
    ) -> Result<Vec<String>, MgError> {
        match self.status {
            ConnectionStatus::Closed => {
                return Err(MgError::new(String::from("Connection is closed")))
            }
            ConnectionStatus::Executing => {
                return Err(MgError::new(String::from(
                    "Connection is already executing",
                )))
            }
            ConnectionStatus::Bad => return Err(MgError::new(String::from("Bad connection"))),
            _ => {}
        }

        if !self.autocommit && !self.in_transaction {
            match self.connection_run_without_results("BEGIN") {
                Ok(()) => self.in_transaction = true,
                Err(err) => return Err(err),
            }
        }

        let c_query = CString::new(query).unwrap();
        let mg_params = match params {
            Some(x) => hash_map_to_mg_map(x),
            None => std::ptr::null_mut(),
        };
        let mut columns = std::ptr::null();
        let status = unsafe {
            bindings::mg_session_run(self.mg_session, c_query.as_ptr(), mg_params, &mut columns)
        };

        if status != 0 {
            self.status = ConnectionStatus::Bad;
            return Err(MgError::new(read_error_message(self.mg_session)));
        }

        self.status = ConnectionStatus::Executing;

        if !self.lazy {
            match self.pull_all() {
                Ok(x) => self.results_iter = Some(x.into_iter()),
                Err(x) => {
                    self.status = ConnectionStatus::Bad;
                    return Err(x);
                }
            }
        }

        Ok(parse_columns(columns))
    }

    pub fn fetchone(&mut self) -> Result<Option<Record>, MgError> {
        match self.status {
            ConnectionStatus::Closed => {
                return Err(MgError::new(String::from("Connection is closed")))
            }
            ConnectionStatus::Ready => {
                return Err(MgError::new(String::from("Connection is not executing")))
            }
            ConnectionStatus::Bad => return Err(MgError::new(String::from("Bad connection"))),
            _ => {}
        }

        match self.lazy {
            true => match self.pull() {
                Ok(res) => match res {
                    Some(x) => Ok(Some(x)),
                    None => {
                        self.status = ConnectionStatus::Ready;
                        Ok(None)
                    }
                },
                Err(err) => {
                    self.status = ConnectionStatus::Bad;
                    Err(err)
                }
            },
            false => match &mut self.results_iter {
                Some(it) => match it.next() {
                    Some(x) => Ok(Some(x)),
                    None => {
                        self.status = ConnectionStatus::Ready;
                        Ok(None)
                    }
                },
                None => panic!(),
            },
        }
    }

    pub fn fetchmany(&mut self, size: Option<u32>) -> Result<Vec<Record>, MgError> {
        let size = match size {
            Some(x) => x,
            None => self.arraysize,
        };

        let mut vec = Vec::new();
        for _i in 0..size {
            match self.fetchone() {
                Ok(record) => match record {
                    Some(x) => vec.push(x),
                    None => break,
                },
                Err(err) => return Err(err),
            }
        }

        Ok(vec)
    }

    pub fn fetchall(&mut self) -> Result<Vec<Record>, MgError> {
        let mut vec = Vec::new();
        loop {
            match self.fetchone() {
                Ok(record) => match record {
                    Some(x) => vec.push(x),
                    None => break,
                },
                Err(err) => return Err(err),
            }
        }

        Ok(vec)
    }

    fn pull(&mut self) -> Result<Option<Record>, MgError> {
        let mut mg_result: *mut bindings::mg_result = std::ptr::null_mut();
        let status = unsafe { bindings::mg_session_pull(self.mg_session, &mut mg_result) };
        let row = unsafe { bindings::mg_result_row(mg_result) };
        match status {
            1 => Ok(Some(Record {
                values: unsafe { mg_list_to_vec(row) },
            })),
            0 => Ok(None),
            _ => Err(MgError::new(read_error_message(self.mg_session))),
        }
    }

    fn pull_all(&mut self) -> Result<Vec<Record>, MgError> {
        let mut res = Vec::new();
        loop {
            match self.pull() {
                Ok(x) => match x {
                    Some(x) => res.push(x),
                    None => break,
                },
                Err(err) => return Err(err),
            }
        }
        Ok(res)
    }

    pub fn commit(&mut self) -> Result<(), MgError> {
        match self.status {
            ConnectionStatus::Closed => {
                return Err(MgError::new(String::from("Connection is closed")))
            }
            ConnectionStatus::Executing => {
                return Err(MgError::new(String::from("Can't commit while executing")))
            }
            ConnectionStatus::Bad => return Err(MgError::new(String::from("Bad connection"))),
            ConnectionStatus::Ready => {}
        }

        if !self.in_transaction {
            return Err(MgError::new(String::from("Not in transaction")));
        }

        match self.connection_run_without_results("COMMIT") {
            Ok(()) => {
                self.in_transaction = false;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn rollback(&mut self) -> Result<(), MgError> {
        match self.status {
            ConnectionStatus::Closed => {
                return Err(MgError::new(String::from("Connection is closed")))
            }
            ConnectionStatus::Executing => {
                return Err(MgError::new(String::from("Can't commit while executing")))
            }
            ConnectionStatus::Bad => return Err(MgError::new(String::from("Bad connection"))),
            ConnectionStatus::Ready => {}
        }

        if !self.in_transaction {
            return Err(MgError::new(String::from("Not in transaction")));
        }

        match self.connection_run_without_results("ROLLBACK") {
            Ok(()) => {
                self.in_transaction = false;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn close(&mut self) {
        match self.status {
            ConnectionStatus::Ready => self.status = ConnectionStatus::Closed,
            ConnectionStatus::Executing => panic!("Connection is executing"),
            _ => {}
        }
    }
}

fn parse_columns(mg_list: *const bindings::mg_list) -> Vec<String> {
    let size = unsafe { bindings::mg_list_size(mg_list) };
    let mut columns: Vec<String> = Vec::new();
    for i in 0..size {
        let mg_value = unsafe { bindings::mg_list_at(mg_list, i) };
        columns.push(mg_value_string(mg_value));
    }
    columns
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
            &c_string_to_string(host, None),
            &c_string_to_string(ip_address, None),
            &c_string_to_string(key_type, None),
            &c_string_to_string(fingerprint, None),
        ) as std::os::raw::c_int
    }
}
