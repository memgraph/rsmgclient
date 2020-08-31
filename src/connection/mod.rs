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
    c_string_to_string, hash_map_to_mg_map, mg_list_to_vec, mg_map_to_hash_map, mg_value_string,
    str_to_c_str, QueryParam, Record, Value,
};
use std::collections::HashMap;
use std::ffi::CString;
use std::vec::IntoIter;

/// Parameters for connecting to database.
///
/// Validation of parameters is performed while calling `Connection::connect`.
///
/// # Examples
///
/// Connecting to localhost database, running on default port 7687.
/// ```
/// use rsmgclient::{ConnectParams, Connection};
/// # use rsmgclient::{MgError};
/// # fn connect() -> Result<(), MgError> {
///
/// let connect_params = ConnectParams {
///     host: Some(String::from("localhost")),
///     ..Default::default()
/// };
///
/// let mut connection = Connection::connect(&connect_params)?;
/// # Ok(()) }
/// ```
pub struct ConnectParams {
    /// Port number to connect to at the server host. Default port is 7687.
    pub port: u16,
    /// DNS resolvable name of host to connect to. Exactly one of host and
    /// address parameters must be specified.
    pub host: Option<String>,
    /// Numeric IP address of host to connect to. This should be in the
    /// standard IPv4 address format. You can also use IPv6 if your machine
    /// supports it. Exactly one of host and address parameters must be
    /// specified.
    pub address: Option<String>,
    /// Username to connect as.
    pub username: Option<String>,
    /// Password to be used if the server demands password authentication.
    pub password: Option<String>,
    /// Alternate name and version of the client to send to server. Default is
    /// "MemgraphBolt/0.1".
    pub client_name: String,
    /// Determines whether a secure SSL TCP/IP connection will be negotiated with
    /// the server. Default value is `SSLMode::Require`.
    pub sslmode: SSLMode,
    /// This parameter specifies the file name of the client SSL certificate.
    /// It is ignored in case an SSL connection is not made.
    pub sslcert: Option<String>,
    /// This parameter specifies the location of the secret key used for the
    /// client certificate. This parameter is ignored in case an SSL connection
    /// is not made.
    pub sslkey: Option<String>,
    /// After performing the SSL handshake, `Connection::connect` will call this
    /// function providing the hostname, IP address, public key type and
    /// fingerprint and user provided data. If the function returns a non-zero
    /// value, SSL connection will be immediately terminated. This can be used
    /// to implement TOFU (trust on first use) mechanism.
    pub trust_callback: Option<*const dyn Fn(&String, &String, &String, &String) -> i32>,
    /// Initial value of `lazy` field, defaults to true, Can be changed using `Connection::set_lazy`.
    pub lazy: bool,
    /// Initial value of `autocommit` field, defaults to false. Can be changed using `Connection::set_autocommit`.
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

/// Determines whether a secure SSL TCP/IP connection will be negotiated
/// with the server.
#[derive(PartialEq)]
pub enum SSLMode {
    /// Only try a non-SSL connection.
    Disable,
    /// Only try a SSL connection.
    Require,
}

/// Encapsulates a database connection.
///
/// # Examples
///
/// ```
/// use rsmgclient::{ConnectParams, Connection};
/// # use rsmgclient::{MgError};
/// # fn execute_query() -> Result<(), MgError> {
///
/// let connect_params = ConnectParams {
///     host: Some(String::from("localhost")),
///     ..Default::default()
/// };
/// let mut connection = Connection::connect(&connect_params)?;
///
/// let query = "CREATE (u:User {name: 'Alice'})-[l:Likes]->(m:Software {name: 'Memgraph'}) RETURN u, l, m";
/// connection.execute(query, None)?;
///
/// let records = connection.fetchall()?;
/// for value in &records[0].values {
///     println!("{}", value);
/// }
///
/// connection.commit()?;
/// # Ok(()) }
/// ```
pub struct Connection {
    mg_session: *mut bindings::mg_session,
    lazy: bool,
    autocommit: bool,
    in_transaction: bool,
    status: ConnectionStatus,
    results_iter: Option<IntoIter<Record>>,
    arraysize: u32,
    summary: Option<HashMap<String, Value>>,
}

/// Representation of current connection status.
#[derive(PartialEq, Debug)]
pub enum ConnectionStatus {
    /// Connection is ready to start executing.
    Ready,
    /// Connection has executed query and is ready to fetch records.
    Executing,
    /// Connection is closed and can no longer be used.
    Closed,
    /// There was an error with current session and connection is no longer usable.
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
    /// Returns whether connection is executing lazily.
    ///
    /// If false, queries are not executed lazily. After running `execute`, records
    /// are immediately pulled.
    ///
    /// If true queries are executed lazily. After running `execute`, records
    /// will only get pulled until fetch functions are called.
    pub fn lazy(&self) -> bool {
        self.lazy
    }

    /// Getter for `autocommit` field.
    ///
    /// If true all queries are automatically committed.
    ///
    /// If false queries are executed inside a transaction. Before executing first query,
    /// `execute` runs `begin` on database. After that user needs to commit or roll back manually, using
    /// `commit` and `rollback` functions.
    pub fn autocommit(&self) -> bool {
        self.autocommit
    }

    /// Getter for `arraysize` field.
    ///
    /// Default amount of rows to get fetched when calling `fetchmany`.
    /// Initial value is `1`.
    pub fn arraysize(&self) -> u32 {
        self.arraysize
    }

    /// Returns whether a connection is currently inside a transaction.
    pub fn in_transaction(&self) -> bool {
        self.in_transaction
    }

    /// Returns current connection status.
    pub fn status(&self) -> &ConnectionStatus {
        &self.status
    }

    /// Returns query summary if it is present.
    ///
    /// Query summary is present after query has completed execution(
    /// all records have been fetched). Executing new query will remove
    /// previous query summary.
    pub fn summary(&self) -> Option<HashMap<String, Value>> {
        match &self.summary {
            Some(x) => Some((*x).clone()),
            None => None,
        }
    }

    /// Setter for `lazy` field.
    ///
    /// # Panics
    ///
    /// Panics if connection is not in a `Ready` status.
    pub fn set_lazy(&mut self, lazy: bool) {
        match self.status {
            ConnectionStatus::Ready => self.lazy = lazy,
            ConnectionStatus::Executing => panic!("Can't set lazy while executing"),
            ConnectionStatus::Bad => panic!("Bad connection"),
            ConnectionStatus::Closed => panic!("Connection is closed"),
        }
    }

    /// Setter for `autocommit` field.
    ///
    /// # Panics
    ///
    /// Panics if connection has pending transaction or connection
    /// is not ready.
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

    /// Setter for `arraysize` field.
    pub fn set_arraysize(&mut self, arraysize: u32) {
        self.arraysize = arraysize;
    }

    /// Creates a connection to database using provided connection parameters.
    ///
    /// Returns `Connection` if connection to database is successfully established,
    /// otherwise returns error with explanation what went wrong.
    ///
    /// # Examples
    ///
    /// ```
    /// use rsmgclient::{ConnectParams, Connection};
    /// # use rsmgclient::{MgError};
    /// # fn connect() -> Result<(), MgError> {
    ///
    /// let connect_params = ConnectParams {
    ///     host: Some(String::from("localhost")),
    ///     ..Default::default()
    /// };
    ///
    /// let mut connection = Connection::connect(&connect_params)?;
    /// # Ok(()) }
    /// ```
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

        Ok(Connection {
            mg_session,
            lazy: param_struct.lazy,
            autocommit: param_struct.autocommit,
            in_transaction: false,
            status: ConnectionStatus::Ready,
            results_iter: None,
            arraysize: 1,
            summary: None,
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

    /// Executes provided query using parameters(if provided) and returns names of columns.
    ///
    /// After execution records need to get fetched using fetch methods.
    /// Connection needs to be in status `Ready`.
    /// Error is returned if connection is not ready, query is invalid
    /// or there was an error in communication with server.
    ///
    /// If connection is not lazy will also fetch and store all records.
    /// If connection has autocommit set to false and is not in a transaction will
    /// also start a transaction.
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

        self.summary = None;

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

    /// Returns next row of query results or None if there is no more data
    /// available.
    ///
    /// Returns error if connection is not in `Executing` status or
    /// if there was an error while pulling record from database.
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

    /// Returns next rows of query results.
    ///
    /// The number of rows to fetch is specified either by `size` or
    /// `arraysize` attribute, `size`(if provided) overrides `arraysize`.
    ///
    /// Returns error if connection is not in `Executing` status or
    /// if there was an error while pulling record from database.
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

    /// Returns all(remaining) rows of query results.
    ///
    /// Returns error if connection is not in `Executing` status or
    /// if there was an error while pulling record from database.
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
            0 => {
                self.summary = Some(mg_map_to_hash_map(unsafe {
                    bindings::mg_result_summary(mg_result)
                }));
                Ok(None)
            }
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

    /// Commit any pending transaction to the database.
    ///
    /// Returns error if there are queries that didn't finish
    /// executing.
    ///
    /// If `autocommit` is set to true or there is no pending transaction
    /// this method does nothing.
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
        if self.autocommit || !self.in_transaction {
            return Ok(());
        }

        match self.connection_run_without_results("COMMIT") {
            Ok(()) => {
                self.in_transaction = false;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Rollback any pending transaction to the database.
    ///
    /// Returns error if there are queries that didn't finish
    /// executing.
    ///
    /// If `autocommit` is set to true or there is no pending transaction
    /// this method does nothing.
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

        if self.autocommit || !self.in_transaction {
            return Ok(());
        }

        match self.connection_run_without_results("ROLLBACK") {
            Ok(()) => {
                self.in_transaction = false;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Closes the connection.
    ///
    /// The connection will be unusable from this point forward. Any operation
    /// on connection will return error.
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
    let fun: &mut &mut dyn Fn(&String, &String, &String, &String) -> i32 = unsafe {
        &mut *(fun_raw
            as *mut &mut dyn for<'r, 's, 't0, 't1> std::ops::Fn(
                &'r std::string::String,
                &'s std::string::String,
                &'t0 std::string::String,
                &'t1 std::string::String,
            ) -> i32)
    };

    unsafe {
        fun(
            &c_string_to_string(host, None),
            &c_string_to_string(ip_address, None),
            &c_string_to_string(key_type, None),
            &c_string_to_string(fingerprint, None),
        ) as std::os::raw::c_int
    }
}

#[cfg(test)]
mod tests;
