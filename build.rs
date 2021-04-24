use cmake::Config;
use std::env;

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=shimexe/shim.cpp");
    println!("cargo:rerun-if-changed=shimexe/CMakeLists.txt");
    println!("cargo:rerun-if-changed=shimexe/cpptoml.h");
    Config::new("shimexe").generator("Ninja").build();
}