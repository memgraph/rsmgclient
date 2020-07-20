use super::bindings;
use super::mg_value::{MgValue, mg_list_to_vec, c_string_to_string};
use std::ffi::{CString};
use super::error::{MgError};

pub struct Connection {
    mg_session: *mut bindings::mg_session,
}

fn read_error_message(mg_session: *mut bindings::mg_session) -> String {
    let c_error_message = unsafe { bindings::mg_session_error(mg_session) };
    unsafe { c_string_to_string(c_error_message) }
}

impl Connection {
    pub fn execute(&self, query: &str) -> Result<Vec<Vec<MgValue>>, MgError> {
        let c_query = CString::new(query).unwrap();
        let mut columns: *const bindings::mg_list = std::ptr::null_mut();
        let mut status = unsafe {
            bindings::mg_session_run(
                self.mg_session, c_query.as_ptr(),
                std::ptr::null(),
                &mut columns,
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
                    0 => {},
                    _ => return Err(MgError::new(read_error_message(self.mg_session))),
                }

                if status != 0 {
                    break;
                }
            }
        };

        Ok(res)
    }
}

pub fn connect(host: &str, port: u16) -> Result<Connection, MgError> {
    let mg_session_params = unsafe {
        bindings::mg_session_params_make()
    };
    let host_c_str = CString::new(host).unwrap();
    unsafe {
        bindings::mg_session_params_set_host(mg_session_params, host_c_str.as_ptr());
        bindings::mg_session_params_set_port(mg_session_params, port);
        bindings::mg_session_params_set_sslmode(mg_session_params, bindings::mg_sslmode_MG_SSLMODE_REQUIRE);
    }

    let mut mg_session: *mut bindings::mg_session = std::ptr::null_mut();
    let status = unsafe { bindings::mg_connect(mg_session_params, &mut mg_session) };
    unsafe { bindings::mg_session_params_destroy(mg_session_params); };

    if status != 0 {
        return Err(MgError::new(read_error_message(mg_session)));
    }

    Ok(Connection {
        mg_session,
    })
}
