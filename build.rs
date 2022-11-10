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
use std::path::{Path, PathBuf};
use std::process::Command;

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
            println!("MacOS detected. We will check if you have either the MacPorts or Homebrew package managers.");
            println!("Checking for MacPorts...");
            let output = Command::new("/usr/bin/command")
                .args(["-v", "port"])
                .output()
                .expect("Failed to execute shell command: '/usr/bin/command -v port'")
                .stdout;

            let port_path = String::from_utf8(output).unwrap();
            if !port_path.is_empty() {
                let port_path = &port_path[..port_path.len() - 1];
                println!(
                    "'port' binary detected at {:?}. We assume MacPorts is installed and is your primary package manager.",
                    &port_path
                );
                let port_binary_path = Path::new(&port_path);

                println!("Checking if the 'openssl' port is installed.");

                let output = String::from_utf8(
                    Command::new(port_path)
                        .args(["installed", "openssl"])
                        .output()
                        .expect("Failed to execute shell command 'port installed openssl'")
                        .stdout,
                )
                .unwrap();

                if output == "None of the specified ports are installed.\n" {
                    panic!("The openssl port does not seem to be installed! Please install it using 'port install openssl'.");
                }

                let openssl_lib_dir = port_binary_path
                    .ancestors()
                    .nth(2)
                    .unwrap()
                    .join("libexec")
                    .join("openssl3")
                    .join("lib");

                // Telling Cargo to tell rustc where to look for the OpenSSL library.
                println!(
                    "cargo:rustc-link-search=native={}",
                    openssl_lib_dir.display()
                );

                // With MacPorts, you don't need to pass in the OPENSSL_ROOT_DIR,
                // OPENSSL_CRYPTO_LIBRARY, and OPENSSL_SSL_LIBRARY options to CMake, PkgConfig
                // should take care of setting those variables.
                Config::new("mgclient").build()
            } else {
                println!("Macports not found.");
                println!("Checking for Homebrew...");

                let output = Command::new("/usr/bin/command")
                    .args(["-v", "brew"])
                    .output()
                    .expect("Failed to execute shell command: '/usr/bin/command -v brew'")
                    .stdout;

                let brew_path = String::from_utf8(output).unwrap();

                if brew_path.is_empty() {
                    println!("Homebrew not found.");
                    panic!("We did not detect either MacPorts or Homebrew on your machine. We cannot proceed.");
                } else {
                    println!("'brew' executable detected at {:?}", &brew_path);
                    println!(
                        "Proceeding with installation assuming Homebrew is your package manager"
                    );
                }

                let path_openssl = if cfg!(target_arch = "aarch64") {
                    "/opt/homebrew/Cellar/openssl@1.1"
                } else {
                    "/usr/local/Cellar/openssl@1.1"
                };
                let mut openssl_dirs = std::fs::read_dir(PathBuf::new().join(path_openssl))
                    .unwrap()
                    .map(|r| r.unwrap().path())
                    .collect::<Vec<PathBuf>>();
                openssl_dirs.sort_by(|a, b| {
                    let a_time = a.metadata().unwrap().modified().unwrap();
                    let b_time = b.metadata().unwrap().modified().unwrap();
                    b_time.cmp(&a_time)
                });
                let openssl_root_path = openssl_dirs[0].clone();
                println!(
                    "cargo:rustc-link-search=native={}",
                    openssl_root_path.join("lib").display()
                );
                let openssl_root = openssl_dirs[0].clone();
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
        }

        _ => cmake::build("mgclient"),
    };

    let mgclient_h = mgclient_out.join("include").join("mgclient.h");
    let mgclient_export_h = mgclient_out.join("include").join("mgclient-export.h");

    // Required because of tests that rely on the C struct fields.
    let mgclient_mgvalue_h = mgclient.join("src").join("mgvalue.h");

    println!("cargo:rerun-if-changed={}", mgclient_h.display());
    println!("cargo:rerun-if-changed={}", mgclient_export_h.display());

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
