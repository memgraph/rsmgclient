// Copyright (c) 2016-2022 Memgraph Ltd. [https://memgraph.com]
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
    QueryParam, Record, Value, c_string_to_string, hash_map_to_mg_map, mg_list_to_vec,
    mg_map_to_hash_map, mg_value_string,
};

use std::collections::HashMap;
use std::ffi::CString;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::vec::IntoIter;

/// Static counter to track the number of active connections.
/// This ensures mg_init() and mg_finalize() are called correctly
/// as they are process-wide operations, not per-connection.
static CONNECTION_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Type alias for SSL trust callback function.
pub type TrustCallback = *const dyn Fn(&String, &String, &String, &String) -> i32;

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
    /// DNS resolvable name of host to connect to. Exactly one of host and address parameters must
    /// be specified.
    pub host: Option<String>,
    /// Numeric IP address of host to connect to. This should be in the standard IPv4 address
    /// format. You can also use IPv6 if your machine supports it. Exactly one of host and address
    /// parameters must be specified.
    pub address: Option<String>,
    /// Username to connect as.
    pub username: Option<String>,
    /// Password to be used if the server demands password authentication.
    pub password: Option<String>,
    /// Alternate name and version of the client to send to server. Default is
    /// "MemgraphBolt/0.1".
    pub client_name: String,
    /// Determines whether a secure SSL TCP/IP connection will be negotiated with the server.
    /// Default value is `SSLMode::Require`.
    pub sslmode: SSLMode,
    /// This parameter specifies the file name of the client SSL certificate. It is ignored in
    /// case an SSL connection is not made.
    pub sslcert: Option<String>,
    /// This parameter specifies the location of the secret key used for the client certificate.
    /// This parameter is ignored in case an SSL connection is not made.
    pub sslkey: Option<String>,
    /// After performing the SSL handshake, `Connection::connect` will call this function providing
    /// the hostname, IP address, public key type and fingerprint and user provided data. If the
    /// function returns a non-zero value, SSL connection will be immediately terminated. This can
    /// be used to implement TOFU (trust on first use) mechanism.
    pub trust_callback: Option<TrustCallback>,
    /// Initial value of `lazy` field, defaults to true, Can be changed using `Connection::set_lazy`.
    pub lazy: bool,
    /// Initial value of `autocommit` field, defaults to false. Can be changed using
    /// `Connection::set_autocommit`.
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
            client_name: String::from("rsmgclient/0.1"),
            sslmode: SSLMode::Disable,
            sslcert: None,
            sslkey: None,
            trust_callback: None,
            lazy: true,
            autocommit: false,
        }
    }
}

/// Determines whether a secure SSL TCP/IP connection will be negotiated with the server.
#[derive(PartialEq, Eq)]
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
    status: ConnectionStatus,
    results_iter: Option<IntoIter<Record>>,
    arraysize: u32,
    summary: Option<HashMap<String, Value>>,
    /// Stored to keep the callback alive for the lifetime of the connection.
    /// mgclient stores the pointer and may call it during SSL operations.
    #[allow(dead_code)]
    trust_callback: Option<Box<TrustCallback>>,
}

/// Representation of current connection status.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum ConnectionStatus {
    /// Connection is ready to start executing.
    Ready,
    /// Connection is in transaction.
    InTransaction,
    /// Connection has executed query and is ready to fetch records.
    Executing,
    /// Connection is in the fetching phase.
    Fetching,
    /// Connection is closed and can no longer be used.
    Closed,
    /// There was an error with current session and connection is no longer usable.
    Bad,
}

fn read_error_message(mg_session: *mut bindings::mg_session) -> String {
    let c_error_message = unsafe { bindings::mg_session_error(mg_session) };
    unsafe { c_string_to_string(c_error_message, None) }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { bindings::mg_session_destroy(self.mg_session) };

        // Decrement the connection counter and finalize only if this was the last connection
        if CONNECTION_COUNT.fetch_sub(1, Ordering::SeqCst) == 1 {
            // This was the last connection, safe to finalize
            Connection::finalize();
        }
    }
}

impl Connection {
    /// Initializes underlying mgclient.
    ///
    /// Should be called at the beginning of each process using the client.
    pub fn init() {
        unsafe {
            bindings::mg_init();
        }
    }

    /// Finalizes underlying mgclient.
    ///
    /// Should be called at the end of each process using the client.
    pub fn finalize() {
        unsafe {
            bindings::mg_finalize();
        }
    }

    /// Returns whether connection is executing lazily.
    ///
    /// If false, queries are not executed lazily. After running `execute`, records are immediately
    /// pulled.
    ///
    /// If true queries are executed lazily. After running `execute`, records will only get pulled
    /// until fetch functions are called.
    pub fn lazy(&self) -> bool {
        self.lazy
    }

    /// Getter for `autocommit` field.
    ///
    /// If true all queries are automatically committed.
    ///
    /// If false queries are executed inside a transaction. Before executing first query, `execute`
    /// runs `begin` on database. After that user needs to commit or roll back manually, using
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

    /// Returns current connection status.
    pub fn status(&self) -> ConnectionStatus {
        self.status
    }

    /// Returns query summary if it is present.
    ///
    /// Query summary is present after query has completed execution( all records have been
    /// fetched). Executing new query will remove previous query summary.
    pub fn summary(&self) -> Option<HashMap<String, Value>> {
        self.summary.as_ref().map(|x| (*x).clone())
    }

    /// Setter for `lazy` field.
    ///
    /// # Panics
    ///
    /// Panics if connection is not in a `Ready` status.
    pub fn set_lazy(&mut self, lazy: bool) {
        match self.status {
            ConnectionStatus::Ready => self.lazy = lazy,
            ConnectionStatus::InTransaction => panic!("Can't set lazy while in transaction"),
            ConnectionStatus::Executing => panic!("Can't set lazy while executing"),
            ConnectionStatus::Fetching => panic!("Can't set lazy while fetching"),
            ConnectionStatus::Bad => panic!("Can't set lazy while connection is bad"),
            ConnectionStatus::Closed => panic!("Can't set lazy while connection is closed"),
        }
    }

    /// Setter for `autocommit` field.
    ///
    /// # Panics
    ///
    /// Panics if connection is not in a `Ready` status.
    pub fn set_autocommit(&mut self, autocommit: bool) {
        match self.status {
            ConnectionStatus::Ready => self.autocommit = autocommit,
            ConnectionStatus::InTransaction => {
                panic!("Can't set autocommit while in transaction")
            }
            ConnectionStatus::Executing => panic!("Can't set autocommit while executing"),
            ConnectionStatus::Fetching => panic!("Can't set autocommit while fetching"),
            ConnectionStatus::Bad => panic!("Can't set autocommit while connection is bad"),
            ConnectionStatus::Closed => panic!("Can't set autocommit while connection is closed"),
        }
    }

    /// Setter for `arraysize` field.
    pub fn set_arraysize(&mut self, arraysize: u32) {
        self.arraysize = arraysize;
    }

    /// Creates a connection to database using provided connection parameters.
    ///
    /// Returns `Connection` if connection to database is successfully established, otherwise
    /// returns error with explanation what went wrong.
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
        // Increment the connection counter and initialize only if this is the first connection
        let prev_count = CONNECTION_COUNT.fetch_add(1, Ordering::SeqCst);
        if prev_count == 0 {
            // First connection, need to initialize
            Connection::init();
        }

        let mg_session_params = unsafe { bindings::mg_session_params_make() };
        if mg_session_params.is_null() {
            // Connection failed, decrement the counter and finalize if needed
            if CONNECTION_COUNT.fetch_sub(1, Ordering::SeqCst) == 1 {
                Connection::finalize();
            }
            return Err(MgError::ffi(
                "Failed to allocate mg_session_params".to_string(),
            ));
        }
        let mut trust_callback_box: Option<Box<TrustCallback>> = None;

        // Create CStrings and keep them alive for the duration of mg_connect
        let c_host = match param_struct.host.as_ref() {
            Some(s) => Some(CString::new(s.as_str()).map_err(|_| MgError::null_byte("host"))?),
            None => None,
        };
        let c_address = match param_struct.address.as_ref() {
            Some(s) => Some(CString::new(s.as_str()).map_err(|_| MgError::null_byte("address"))?),
            None => None,
        };
        let c_username = match param_struct.username.as_ref() {
            Some(s) => Some(CString::new(s.as_str()).map_err(|_| MgError::null_byte("username"))?),
            None => None,
        };
        let c_password = match param_struct.password.as_ref() {
            Some(s) => Some(CString::new(s.as_str()).map_err(|_| MgError::null_byte("password"))?),
            None => None,
        };
        let c_client_name = CString::new(param_struct.client_name.as_str())
            .map_err(|_| MgError::null_byte("client_name"))?;
        let c_sslcert = match param_struct.sslcert.as_ref() {
            Some(s) => Some(CString::new(s.as_str()).map_err(|_| MgError::null_byte("sslcert"))?),
            None => None,
        };
        let c_sslkey = match param_struct.sslkey.as_ref() {
            Some(s) => Some(CString::new(s.as_str()).map_err(|_| MgError::null_byte("sslkey"))?),
            None => None,
        };

        unsafe {
            if let Some(ref x) = c_host {
                bindings::mg_session_params_set_host(mg_session_params, x.as_ptr())
            }
            bindings::mg_session_params_set_port(mg_session_params, param_struct.port);
            if let Some(ref x) = c_address {
                bindings::mg_session_params_set_address(mg_session_params, x.as_ptr())
            }
            if let Some(ref x) = c_username {
                bindings::mg_session_params_set_username(mg_session_params, x.as_ptr())
            }
            if let Some(ref x) = c_password {
                bindings::mg_session_params_set_password(mg_session_params, x.as_ptr())
            }
            bindings::mg_session_params_set_user_agent(mg_session_params, c_client_name.as_ptr());
            bindings::mg_session_params_set_sslmode(
                mg_session_params,
                // Bindings struct is not used because on Linux bindgen
                // generates u32, while on Windows i32 type is generated.
                match param_struct.sslmode {
                    SSLMode::Disable => 0,
                    SSLMode::Require => 1,
                },
            );
            if let Some(ref x) = c_sslcert {
                bindings::mg_session_params_set_sslcert(mg_session_params, x.as_ptr())
            }
            if let Some(ref x) = c_sslkey {
                bindings::mg_session_params_set_sslkey(mg_session_params, x.as_ptr())
            }
            if let Some(x) = &param_struct.trust_callback {
                let callback_box = Box::new(*x);
                let trust_callback_ptr = Box::into_raw(callback_box);

                bindings::mg_session_params_set_trust_data(
                    mg_session_params,
                    trust_callback_ptr as *mut ::std::os::raw::c_void,
                );
                bindings::mg_session_params_set_trust_callback(
                    mg_session_params,
                    Some(trust_callback_wrapper),
                );

                // Store the callback box for later (will be owned by Connection)
                // SAFETY: We just created this raw pointer from Box::into_raw above
                trust_callback_box = Some(Box::from_raw(trust_callback_ptr));
            }
        }

        let mut mg_session: *mut bindings::mg_session = std::ptr::null_mut();
        let status = unsafe { bindings::mg_connect(mg_session_params, &mut mg_session) };
        unsafe {
            bindings::mg_session_params_destroy(mg_session_params);
        };

        if status != 0 {
            // Connection failed, decrement the counter and finalize if needed
            if CONNECTION_COUNT.fetch_sub(1, Ordering::SeqCst) == 1 {
                Connection::finalize();
            }
            return Err(MgError::connection(read_error_message(mg_session)));
        }

        Ok(Connection {
            mg_session,
            lazy: param_struct.lazy,
            autocommit: param_struct.autocommit,
            status: ConnectionStatus::Ready,
            results_iter: None,
            arraysize: 1,
            summary: None,
            trust_callback: trust_callback_box,
        })
    }

    /// Fully Executes provided query but doesn't return any results even if they exist.
    pub fn execute_without_results(&mut self, query: &str) -> Result<(), MgError> {
        // Allocate the C string without leaking. Keep it alive for the duration of the FFI call.
        let c_query = CString::new(query).map_err(|_| MgError::null_byte("query"))?;

        match unsafe {
            bindings::mg_session_run(
                self.mg_session,
                c_query.as_ptr(), // no leak: pointer valid during this call
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        } {
            0 => {
                self.status = ConnectionStatus::Executing;
            }
            _ => {
                self.status = ConnectionStatus::Bad;
                return Err(MgError::query(read_error_message(self.mg_session)));
            }
        }

        match unsafe { bindings::mg_session_pull(self.mg_session, std::ptr::null_mut()) } {
            0 => {
                self.status = ConnectionStatus::Fetching;
            }
            _ => {
                self.status = ConnectionStatus::Bad;
                return Err(MgError::query(read_error_message(self.mg_session)));
            }
        }

        loop {
            let mut result = std::ptr::null_mut();
            match unsafe { bindings::mg_session_fetch(self.mg_session, &mut result) } {
                1 => {
                    continue;
                }
                0 => {
                    self.status = ConnectionStatus::Ready;
                    return Ok(());
                }
                _ => {
                    self.status = ConnectionStatus::Bad;
                    return Err(MgError::query(read_error_message(self.mg_session)));
                }
            };
        }
    }

    /// Executes provided query using parameters (if provided) and returns names of columns.
    ///
    /// After execution records need to get fetched using fetch methods. Connection needs to be in
    /// status `Ready` or `InTransaction`. Error is returned if connection is not ready, query is
    /// invalid or there was an error in communication with server.
    ///
    /// If connection is not lazy will also fetch and store all records. If connection has
    /// autocommit set to false and is not in a transaction will also start a transaction.
    pub fn execute(
        &mut self,
        query: &str,
        params: Option<&HashMap<String, QueryParam>>,
    ) -> Result<Vec<String>, MgError> {
        match self.status {
            ConnectionStatus::Ready => {}
            ConnectionStatus::InTransaction => {}
            ConnectionStatus::Executing => {
                return Err(MgError::invalid_state("execute", "already executing"));
            }
            ConnectionStatus::Fetching => {
                return Err(MgError::invalid_state("execute", "fetching"));
            }
            ConnectionStatus::Closed => {
                return Err(MgError::invalid_state("execute", "connection closed"));
            }
            ConnectionStatus::Bad => {
                return Err(MgError::invalid_state("execute", "bad connection"));
            }
        }

        if !self.autocommit && self.status == ConnectionStatus::Ready {
            match self.execute_without_results("BEGIN") {
                Ok(()) => self.status = ConnectionStatus::InTransaction,
                Err(err) => return Err(err),
            }
        }

        self.summary = None;

        let c_query = CString::new(query).map_err(|_| MgError::null_byte("query"))?;
        let mg_params = match params {
            Some(x) => hash_map_to_mg_map(x),
            None => std::ptr::null_mut(),
        };
        let mut columns = std::ptr::null();
        let status = unsafe {
            bindings::mg_session_run(
                self.mg_session,
                c_query.as_ptr(),
                mg_params,
                std::ptr::null_mut(),
                &mut columns,
                std::ptr::null_mut(),
            )
        };

        // Clean up the parameter map - mgclient has copied the data
        if !mg_params.is_null() {
            unsafe { bindings::mg_map_destroy(mg_params) };
        }

        if status != 0 {
            self.status = ConnectionStatus::Bad;
            return Err(MgError::query(read_error_message(self.mg_session)));
        }

        self.status = ConnectionStatus::Executing;

        if !self.lazy {
            match self.pull_and_fetch_all() {
                Ok(x) => self.results_iter = Some(x.into_iter()),
                Err(x) => {
                    self.status = ConnectionStatus::Bad;
                    return Err(x);
                }
            }
        }

        Ok(parse_columns(columns))
    }

    /// Returns next row of query results or None if there is no more data available.
    ///
    /// Returns error if connection is not in `Executing` status or if there was an error while
    /// pulling record from database.
    pub fn fetchone(&mut self) -> Result<Option<Record>, MgError> {
        match self.status {
            ConnectionStatus::Ready => {
                return Err(MgError::invalid_state("fetchone", "ready"));
            }
            ConnectionStatus::InTransaction => {
                return Err(MgError::invalid_state("fetchone", "in transaction"));
            }
            ConnectionStatus::Executing => {}
            ConnectionStatus::Fetching => {}
            ConnectionStatus::Closed => {
                return Err(MgError::invalid_state("fetchone", "connection closed"));
            }
            ConnectionStatus::Bad => {
                return Err(MgError::invalid_state("fetchone", "bad connection"));
            }
        }

        match self.lazy {
            true => {
                if self.status == ConnectionStatus::Executing {
                    match self.pull(1) {
                        Ok(_) => {
                            // The state update is already done in the pull.
                        }
                        Err(err) => {
                            self.status = ConnectionStatus::Bad;
                            return Err(err);
                        }
                    }
                }

                // Fetch the record or summary
                match self.fetch() {
                    Ok((Some(x), None)) => {
                        // Got a record, fetch summary to check has_more
                        match self.fetch()? {
                            (None, Some(has_more)) => {
                                if has_more {
                                    self.status = ConnectionStatus::Executing;
                                }
                                // If has_more is false, leave status as Fetching
                            }
                            _ => {
                                // If we don't get a summary, stay in Fetching state
                            }
                        }
                        Ok(Some(x))
                    }
                    Ok((None, Some(has_more))) => {
                        // Got summary (no more local records)
                        if has_more {
                            self.status = ConnectionStatus::Executing;
                        } else {
                            self.status = if self.autocommit {
                                ConnectionStatus::Ready
                            } else {
                                ConnectionStatus::InTransaction
                            };
                        }
                        Ok(None)
                    }
                    Ok(_) => {
                        // Unexpected case
                        self.status = if self.autocommit {
                            ConnectionStatus::Ready
                        } else {
                            ConnectionStatus::InTransaction
                        };
                        Ok(None)
                    }
                    Err(_) => {
                        // If fetch fails (e.g., "called fetch while not executing"),
                        // it means no more records, finalize the transaction
                        self.status = if self.autocommit {
                            ConnectionStatus::Ready
                        } else {
                            ConnectionStatus::InTransaction
                        };
                        Ok(None)
                    }
                }
            }
            false => match self.next_record() {
                Some(x) => Ok(Some(x)),
                None => {
                    self.status = if self.autocommit {
                        ConnectionStatus::Ready
                    } else {
                        ConnectionStatus::InTransaction
                    };
                    Ok(None)
                }
            },
        }
    }

    fn next_record(&mut self) -> Option<Record> {
        if let Some(iter) = self.results_iter.as_mut() {
            iter.next()
        } else {
            None
        }
    }

    /// Returns next rows of query results.
    ///
    /// The number of rows to fetch is specified either by `size` or `arraysize` attribute,
    /// `size`(if provided) overrides `arraysize`.
    ///
    /// Returns error if connection is not in `Executing` status or if there was an error while
    /// pulling record from database.
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
    /// Returns error if connection is not in `Executing` status or if there was an error while
    /// pulling record from database.
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

    fn pull(&mut self, n: i64) -> Result<(), MgError> {
        match self.status {
            ConnectionStatus::Ready => {
                return Err(MgError::invalid_state("pull", "ready"));
            }
            ConnectionStatus::InTransaction => {
                return Err(MgError::invalid_state("pull", "in transaction"));
            }
            ConnectionStatus::Executing => {}
            ConnectionStatus::Fetching => {
                return Err(MgError::invalid_state("pull", "fetching"));
            }
            ConnectionStatus::Closed => {
                return Err(MgError::invalid_state("pull", "connection closed"));
            }
            ConnectionStatus::Bad => {
                return Err(MgError::invalid_state("pull", "bad connection"));
            }
        }

        let pull_status = match n {
            0 => unsafe { bindings::mg_session_pull(self.mg_session, std::ptr::null_mut()) },
            _ => {
                // Create NUL-terminated C string outside unsafe block to ensure proper lifetime
                let n_key = CString::new("n").expect("'n' is a valid C string");
                unsafe {
                    let mg_map = bindings::mg_map_make_empty(1);
                    if mg_map.is_null() {
                        self.status = ConnectionStatus::Bad;
                        return Err(MgError::ffi("Failed to allocate pull map".to_string()));
                    }
                    let mg_int = bindings::mg_value_make_integer(n);
                    if mg_int.is_null() {
                        self.status = ConnectionStatus::Bad;
                        bindings::mg_map_destroy(mg_map);
                        return Err(MgError::ffi(
                            "Failed to allocate pull map integer value".to_string(),
                        ));
                    }
                    if bindings::mg_map_insert(mg_map, n_key.as_ptr(), mg_int) != 0 {
                        self.status = ConnectionStatus::Bad;
                        bindings::mg_map_destroy(mg_map);
                        bindings::mg_value_destroy(mg_int);
                        return Err(MgError::ffi("Failed to insert into pull map".to_string()));
                    }
                    let status = bindings::mg_session_pull(self.mg_session, mg_map);
                    bindings::mg_map_destroy(mg_map);
                    status
                }
            }
        };

        match pull_status {
            0 => {
                self.status = ConnectionStatus::Fetching;
                Ok(())
            }
            _ => {
                self.status = ConnectionStatus::Bad;
                Err(MgError::query(read_error_message(self.mg_session)))
            }
        }
    }

    /// Maybe returns Record and has_more flag.
    fn fetch(&mut self) -> Result<(Option<Record>, Option<bool>), MgError> {
        match self.status {
            ConnectionStatus::Ready => {
                return Err(MgError::invalid_state("fetch", "ready"));
            }
            ConnectionStatus::InTransaction => {
                return Err(MgError::invalid_state("fetch", "in transaction"));
            }
            ConnectionStatus::Executing => {
                return Err(MgError::invalid_state("fetch", "executing"));
            }
            ConnectionStatus::Fetching => {}
            ConnectionStatus::Closed => {
                return Err(MgError::invalid_state("fetch", "connection closed"));
            }
            ConnectionStatus::Bad => {
                return Err(MgError::invalid_state("fetch", "bad connection"));
            }
        }

        let mut mg_result: *mut bindings::mg_result = std::ptr::null_mut();
        let fetch_status = unsafe { bindings::mg_session_fetch(self.mg_session, &mut mg_result) };
        match fetch_status {
            1 => unsafe {
                let row = bindings::mg_result_row(mg_result);
                Ok((
                    Some(Record {
                        values: mg_list_to_vec(row),
                    }),
                    None,
                ))
            },
            0 => unsafe {
                let mg_summary = bindings::mg_result_summary(mg_result);
                // "has_more" is a known constant string with no null bytes
                let c_has_more = CString::new("has_more").expect("'has_more' is a valid C string");
                let mg_has_more = bindings::mg_map_at(mg_summary, c_has_more.as_ptr());
                let has_more = bindings::mg_value_bool(mg_has_more) != 0;
                self.summary = Some(mg_map_to_hash_map(mg_summary));
                Ok((None, Some(has_more)))
            },
            _ => Err(MgError::query(read_error_message(self.mg_session))),
        }
    }

    fn pull_and_fetch_all(&mut self) -> Result<Vec<Record>, MgError> {
        let mut res = Vec::new();
        match self.pull(0) {
            Ok(_) => loop {
                let x = self.fetch()?;
                match x {
                    (Some(x), _) => res.push(x),
                    (None, _) => break,
                }
            },
            Err(err) => return Err(err),
        }
        Ok(res)
    }

    /// Commit any pending transaction to the database.
    ///
    /// Returns error if there are queries that didn't finish executing.
    ///
    /// If `autocommit` is set to true or there is no pending transaction this method does nothing.
    pub fn commit(&mut self) -> Result<(), MgError> {
        match self.status {
            ConnectionStatus::Ready => {}
            ConnectionStatus::InTransaction => {}
            ConnectionStatus::Executing => {
                return Err(MgError::invalid_state("commit", "executing"));
            }
            ConnectionStatus::Fetching => {
                return Err(MgError::invalid_state("commit", "fetching"));
            }
            ConnectionStatus::Closed => {
                return Err(MgError::invalid_state("commit", "connection closed"));
            }
            ConnectionStatus::Bad => {
                return Err(MgError::invalid_state("commit", "bad connection"));
            }
        }

        if self.autocommit || self.status != ConnectionStatus::InTransaction {
            return Ok(());
        }

        match self.execute_without_results("COMMIT") {
            Ok(()) => {
                self.status = ConnectionStatus::Ready;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Rollback any pending transaction to the database.
    ///
    /// Returns error if there are queries that didn't finish executing.
    ///
    /// If `autocommit` is set to true or there is no pending transaction this method does nothing.
    pub fn rollback(&mut self) -> Result<(), MgError> {
        match self.status {
            ConnectionStatus::Ready => {
                return Err(MgError::invalid_state("rollback", "not in transaction"));
            }
            ConnectionStatus::InTransaction => {}
            ConnectionStatus::Executing => {
                return Err(MgError::invalid_state("rollback", "executing"));
            }
            ConnectionStatus::Fetching => {
                return Err(MgError::invalid_state("rollback", "fetching"));
            }
            ConnectionStatus::Closed => {
                return Err(MgError::invalid_state("rollback", "connection closed"));
            }
            ConnectionStatus::Bad => {
                return Err(MgError::invalid_state("rollback", "bad connection"));
            }
        }

        if self.autocommit {
            return Ok(());
        }

        match self.execute_without_results("ROLLBACK") {
            Ok(()) => {
                self.status = ConnectionStatus::Ready;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Closes the connection.
    ///
    /// The connection will be unusable from this point forward. Any operation on connection will
    /// return error.
    pub fn close(&mut self) {
        match self.status {
            ConnectionStatus::Ready => self.status = ConnectionStatus::Closed,
            ConnectionStatus::InTransaction => self.status = ConnectionStatus::Closed,
            ConnectionStatus::Executing => panic!("Can't close while executing"),
            ConnectionStatus::Fetching => panic!("Can't close while fetching"),
            ConnectionStatus::Closed => {}
            ConnectionStatus::Bad => panic!("Can't closed a bad connection"),
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
