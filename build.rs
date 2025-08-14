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
use std::error::Error;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(PartialEq)]
enum HostType {
    Linux,
    MacOS,
    Windows,
    Unknown,
}

#[derive(Debug)]
enum BuildError {
    IoError(String),
    OpenSSL(String),
    Unknown(String),
}

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::IoError(msg) => write!(f, "Failed to execute shell command: {}", msg),
            BuildError::OpenSSL(msg) => write!(f, "OpenSSL Error: {}", msg),
            BuildError::Unknown(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl Error for BuildError {}

// NOTE: The code here is equivalent to [rust-openssl](https://github.com/sfackler/rust-openssl).
// NOTE: We have to build mgclient and link the rust binary with the same SSL and Crypto libs.

fn build_mgclient_macos() -> Result<PathBuf, BuildError> {
    println!("MacOS detected. We will check if you have either the MacPorts or Homebrew package managers.");
    println!("Checking for MacPorts...");
    let output = Command::new("/usr/bin/command")
        .args(["-v", "port"])
        .output()
        .map_err(|err| BuildError::IoError(format!("'/usr/bin/command -v port': {}", err)))?
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
        let path = Config::new("mgclient").build();
        Ok(path)
    } else {
        println!("Macports not found.");
        println!("Checking for Homebrew...");
        let output = Command::new("/usr/bin/command")
            .args(["-v", "brew"])
            .output()
            .map_err(|err| BuildError::IoError(format!("'/usr/bin/command -v brew': {}", err)))?
            .stdout;
        let brew_path = String::from_utf8(output).unwrap();
        if brew_path.is_empty() {
            println!("Homebrew not found.");
            BuildError::Unknown(
                "We did not detect either MacPorts or Homebrew on your machine. We cannot proceed."
                    .to_string(),
            );
        } else {
            println!("'brew' executable detected at {:?}", &brew_path);
            println!("Proceeding with installation assuming Homebrew is your package manager");
        }

        let path_openssl = if cfg!(target_arch = "aarch64") {
            "/opt/homebrew/Cellar/openssl@3"
        } else {
            "/usr/local/Cellar/openssl@3"
        };
        println!("Found OpenSSL at path: {}", path_openssl);

        let mut openssl_dirs = std::fs::read_dir(PathBuf::new().join(path_openssl))
            .map_err(|err| {
                BuildError::OpenSSL(format!("Failed to read OpenSSL directory: '{}'", err))
            })?
            .filter_map(|r| match r {
                Ok(entry) => Some(entry.path()),
                Err(err) => {
                    // Return the error as a BuildError
                    Err(BuildError::OpenSSL(format!(
                        "Failed to read directory entry: '{}'",
                        err
                    )))
                    .ok()
                }
            })
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
        let path = Config::new("mgclient")
            .define("OPENSSL_ROOT_DIR", format!("{}", openssl_root.display()))
            .define(
                "OPENSSL_INCLUDE_DIR",
                format!("{}", openssl_root.join("include").display()),
            )
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
            .build();

        Ok(path)
    }
}

fn build_mgclient_linux() -> Result<PathBuf, BuildError> {
    let path = Config::new("mgclient").build();
    Ok(path)
}

fn build_mgclient_windows() -> Result<PathBuf, BuildError> {
    let openssl_lib_dir = PathBuf::from(
        std::env::var("OPENSSL_LIB_DIR")
            .unwrap_or_else(|_| "C:\\Program Files\\OpenSSL-Win64\\lib".to_string()),
    );
    let openssl_include_dir = PathBuf::from(
        std::env::var("OPENSSL_INCLUDE_DIR")
            .unwrap_or_else(|_| "C:\\Program Files\\OpenSSL-Win64\\include".to_string()),
    );
    let openssl_root_dir = openssl_lib_dir.parent().unwrap_or(&openssl_lib_dir);

    println!(
        "cargo:rustc-link-search=native={}",
        openssl_lib_dir.display()
    );

    // Check if we're using vcpkg (static libraries)
    let is_vcpkg = openssl_lib_dir.to_string_lossy().contains("vcpkg");
    let (crypto_lib, ssl_lib) = if is_vcpkg {
        // vcpkg uses different library names for static builds
        (
            format!("{}\\libcrypto.lib", openssl_lib_dir.display()),
            format!("{}\\libssl.lib", openssl_lib_dir.display()),
        )
    } else {
        // Standard OpenSSL installation
        (
            format!("{}\\libcrypto.lib", openssl_lib_dir.display()),
            format!("{}\\libssl.lib", openssl_lib_dir.display()),
        )
    };

    let path = Config::new("mgclient")
        .define(
            "OPENSSL_ROOT_DIR",
            format!("{}", openssl_root_dir.display()),
        )
        .define(
            "OPENSSL_INCLUDE_DIR",
            format!("{}", openssl_include_dir.display()),
        )
        .define("OPENSSL_CRYPTO_LIBRARY", crypto_lib)
        .define("OPENSSL_SSL_LIBRARY", ssl_lib)
        .build();

    Ok(path)
}

fn main() -> Result<(), BuildError> {
    let host_type = if cfg!(target_os = "linux") {
        HostType::Linux
    } else if cfg!(target_os = "windows") {
        HostType::Windows
    } else if cfg!(target_os = "macos") {
        HostType::MacOS
    } else {
        HostType::Unknown
    };

    let mgclient = PathBuf::new().join("mgclient");
    let mgclient_out = match host_type {
        HostType::Windows => build_mgclient_windows(),
        HostType::MacOS => build_mgclient_macos(),
        HostType::Linux => build_mgclient_linux(),
        HostType::Unknown => Err(BuildError::Unknown("Unknown operating system".to_string())),
    }?;

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

    let lib_dir = if Path::new(&mgclient_out.join("lib64")).exists() {
        "lib64"
    } else {
        "lib"
    };
    println!(
        "cargo:rustc-link-search=native={}",
        mgclient_out.join(lib_dir).display()
    );
    println!("cargo:rustc-link-lib=static=mgclient");
    // If the following part of the code is pushed inside build_mgclient_xzy, linking is not done
    // properly.
    match host_type {
        HostType::Linux => {
            println!("cargo:rustc-link-lib=dylib=crypto");
            println!("cargo:rustc-link-lib=dylib=ssl");
        }
        HostType::Windows => {
            println!("cargo:rustc-link-lib=dylib=libcrypto");
            println!("cargo:rustc-link-lib=dylib=libssl");
        }
        HostType::MacOS => {
            println!("cargo:rustc-link-lib=dylib=crypto");
            println!("cargo:rustc-link-lib=dylib=ssl");
        }
        HostType::Unknown => panic!("Unknown operating system"),
    }

    Ok(())
}
