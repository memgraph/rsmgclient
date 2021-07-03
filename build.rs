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

extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let mgclient = PathBuf::new().join("mgclient");
    let mgclient_out = cmake::build("mgclient");

    let mgclient_h = mgclient_out.join("include").join("mgclient.h");
    let mgclient_export_h = mgclient_out.join("include").join("mgclient-export.h");
    // Required because of tests that rely on the C struct fields.
    let mgclient_mgvalue_h = mgclient.join("src").join("mgvalue.h");

    println!(
        "{}",
        format!("{}{}", "cargo:rerun-if-changed=", mgclient_h.display())
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "cargo:rerun-if-changed=",
            mgclient_export_h.display()
        )
    );

    let bindings = bindgen::Builder::default()
        .header(format!("{}", mgclient_h.display()))
        .header(format!("{}", mgclient_export_h.display()))
        .header(format!("{}", mgclient_mgvalue_h.display()))
        .clang_arg(format!("-I{}", mgclient_out.join("include").display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!(
        "cargo:rustc-link-search=native={}",
        mgclient_out.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=mgclient");
    println!("cargo:rustc-link-lib=dylib=crypto");
    println!("cargo:rustc-link-lib=dylib=ssl");
}
