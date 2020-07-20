fn main() {
    // Tell cargo to tell rustc to link the system mgclient
    // shared library.
    println!("cargo:rustc-link-lib=mgclient");
}
