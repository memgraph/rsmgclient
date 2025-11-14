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

use thiserror::Error;

/// Error types returned by the Memgraph client.
#[derive(Debug, Error)]
pub enum MgError {
    /// Connection-related errors
    #[error("Connection error: {0}")]
    Connection(String),

    /// Query execution errors
    #[error("Query execution error: {0}")]
    QueryExecution(String),

    /// Invalid parameter errors
    #[error("Invalid parameter '{parameter}': {reason}")]
    InvalidParameter { parameter: String, reason: String },

    /// Null byte in string parameter
    #[error("Parameter '{parameter}' contains null byte")]
    NullByte { parameter: String },

    /// Invalid timestamp
    #[error("Invalid timestamp values")]
    InvalidTimestamp,

    /// Invalid connection state for operation
    #[error("Invalid connection state: {operation} cannot be called while {state}")]
    InvalidState { operation: String, state: String },

    /// FFI-related errors
    #[error("FFI error: {0}")]
    Ffi(String),
}

impl MgError {
    /// Creates a new connection error.
    pub fn connection(message: impl Into<String>) -> Self {
        MgError::Connection(message.into())
    }

    /// Creates a new query execution error.
    pub fn query(message: impl Into<String>) -> Self {
        MgError::QueryExecution(message.into())
    }

    /// Creates a new invalid parameter error.
    pub fn invalid_parameter(parameter: impl Into<String>, reason: impl Into<String>) -> Self {
        MgError::InvalidParameter {
            parameter: parameter.into(),
            reason: reason.into(),
        }
    }

    /// Creates a new null byte error.
    pub fn null_byte(parameter: impl Into<String>) -> Self {
        MgError::NullByte {
            parameter: parameter.into(),
        }
    }

    /// Creates a new invalid state error.
    pub fn invalid_state(operation: impl Into<String>, state: impl Into<String>) -> Self {
        MgError::InvalidState {
            operation: operation.into(),
            state: state.into(),
        }
    }

    /// Creates a new FFI error.
    pub fn ffi(message: impl Into<String>) -> Self {
        MgError::Ffi(message.into())
    }

    /// Legacy constructor for backward compatibility during migration.
    #[deprecated(note = "Use specific error constructors instead")]
    pub fn new(message: String) -> Self {
        MgError::Connection(message)
    }
}
