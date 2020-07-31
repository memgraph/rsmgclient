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

use super::connection::Connection;
use super::error::MgError;
use super::value::{
    c_string_to_string, hash_map_to_mg_map, mg_list_to_vec, mg_value_string, str_to_c_str,
    QueryParam, Record, Value,
};
use std::collections::HashMap;

#[derive(PartialEq)]
pub enum CursorStatus {
    Ready,
    Executing,
    Closed,
}

pub struct Cursor<'a> {
    pub(crate) connection: &'a mut Connection,
    pub(crate) status: CursorStatus,
    pub(crate) record: Option<Record>,
    pub(crate) cached_records: Vec<Record>,
    pub(crate) arraysize: u32,
    pub(crate) rownumber: i32,
    pub(crate) columns: Option<Vec<String>>,
}

impl<'a> Cursor<'a> {
    pub fn get_columns(&'a self) -> Result<&'a Vec<String>, MgError> {
        match &self.columns {
            Some(x) => Ok(x),
            None => Err(MgError::new(String::from("Columns not available"))),
        }
    }

    pub fn execute(
        &mut self,
        query: &String,
        params: Option<&HashMap<String, QueryParam>>,
    ) -> Result<(), MgError> {
        match self.status {
            CursorStatus::Closed => return Err(MgError::new(String::from("Cursor is closed"))),
            CursorStatus::Executing => {
                return Err(MgError::new(String::from("Cursor is executing")))
            }
            _ => {}
        }

        self.reset();

        self.columns = match self.connection.run(&query, params) {
            Ok(x) => Some(x),
            Err(x) => return Err(x),
        };

        self.status = CursorStatus::Executing;
        if !self.connection.lazy {
            match self.connection.pull_all() {
                Ok(x) => self.cached_records = x,
                Err(x) => return Err(x),
            }
        }

        Ok(())
    }

    pub fn fetchone(&mut self) -> Result<Option<&Record>, MgError> {
        if self.status != CursorStatus::Executing {
            return Err(MgError::new(String::from("Invalid cursor status")));
        }

        if !self.connection.lazy {
            self.rownumber += 1;
            let len = self.cached_records.len();
            return Ok(match self.rownumber {
                x if x < len as i32 => Some(&self.cached_records[x as usize]),
                _ => {
                    self.status = CursorStatus::Ready;
                    None
                }
            });
        }

        match self.connection.pull() {
            Ok(x) => Ok(match x {
                Some(x) => {
                    self.rownumber += 1;
                    self.record = Some(x);
                    self.record.as_ref()
                }
                None => {
                    self.status = CursorStatus::Ready;
                    None
                }
            }),
            Err(x) => Err(x),
        }
    }

    pub fn fetchmany(&mut self, size: Option<u32>) -> Result<&[Record], MgError> {
        if self.status != CursorStatus::Executing {
            return Err(MgError::new(String::from("Invalid cursor status")));
        }

        let amount = match size {
            Some(x) => x,
            None => self.arraysize,
        };

        if !self.connection.lazy {
            let start = (self.rownumber + 1) as usize;
            let end = std::cmp::min(
                self.cached_records.len() as i32,
                (self.rownumber + 1 + amount as i32),
            ) as usize;
            let res = Ok(&self.cached_records[start..end]);
            self.rownumber = (end - 1) as i32;
            if end - start < amount as usize {
                self.status = CursorStatus::Ready;
            }
            return res;
        }

        self.cached_records.clear();
        for _i in 0..amount {
            match self.connection.pull() {
                Ok(x) => match x {
                    Some(x) => {
                        self.rownumber += 1;
                        self.cached_records.push(x)
                    }
                    None => {
                        self.status = CursorStatus::Ready;
                        break;
                    }
                },
                Err(x) => return Err(x),
            }
        }

        Ok(&self.cached_records)
    }

    pub fn fetchall(&mut self) -> Result<&Vec<Record>, MgError> {
        if self.status != CursorStatus::Executing {
            return Err(MgError::new(String::from("Invalid state")));
        }

        if !self.connection.lazy {
            self.status = CursorStatus::Ready;
            return Ok(&self.cached_records);
        }

        match self.connection.pull_all() {
            Ok(x) => self.cached_records = x,
            Err(x) => return Err(x),
        }

        Ok(&self.cached_records)
    }

    fn reset(&mut self) {
        self.cached_records.clear();
        self.record = None;
        self.rownumber = -1;
        self.columns = None;
        self.status = CursorStatus::Ready;
    }

    pub fn close(&mut self) -> Result<(), MgError> {
        if self.status == CursorStatus::Executing {
            return Err(MgError::new(String::from(
                "Cannot close cursor while executing",
            )));
        }
        self.reset();
        self.status = CursorStatus::Closed;
        Ok(())
    }
}
