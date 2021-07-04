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

use cmake::Config;
use std::env;
use std::path::PathBuf;

#[derive(PartialEq)]
enum HostType {
    Linux,
    MacOS,
    Windows,
    Unknown,
}

fn main() {
    let host_type = if cfg!(target_os = "linux") {
        HostType::Linux
    } else if cfg!(windows) {
        HostType::Windows
    } else if cfg!(target_os = "macos") {
        HostType::MacOS
    } else {
        HostType::Unknown
    };

    let mgclient = PathBuf::new().join("mgclient");
    let mgclient_out = match host_type {
        // Please checkout what are Windows requirements to compile
        // https://github.com/memgraph/mgclient.
        // While installing Rust please select x86_64-pc-windows-gnu
        // as the host triplet (custom installation step is required).
        HostType::Windows => Config::new("mgclient")
            .target("windows-gnu")
            .generator("MinGW Makefiles")
            .build(),

        HostType::MacOS => {
            let mut openssl_dirs =
                std::fs::read_dir(PathBuf::new().join("/usr/local/Cellar/openssl@1.1"))
                    .unwrap()
                    .map(|r| r.unwrap().path())
                    .collect::<Vec<PathBuf>>();
            openssl_dirs.sort_by(|a, b| {
                let a_time = a.metadata().unwrap().modified().unwrap();
                let b_time = b.metadata().unwrap().modified().unwrap();
                b_time.cmp(&a_time)
            });
            let openssl_root = openssl_dirs[0].clone();
            println!(
                "cargo:rustc-link-search=native={}",
                openssl_root.join("lib").display()
            );
            Config::new("mgclient")
                .define("OPENSSL_ROOT_DIR", format!("{}", openssl_root.display()))
                .define(
                    "OPENSSL_CRYPTO_LIBRARY",
                    format!(
                        "{}",
                        openssl_root.join("lib").join("libcrypto.dylib").display()
                    ),
                )
                .define(
                    "OPENSSL_SSL_LIBRARY",
                    format!(
                        "{}",
                        openssl_root.join("lib").join("libssl.dylib").display()
                    ),
                )
                .build()
        }

        _ => cmake::build("mgclient"),
    };

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
