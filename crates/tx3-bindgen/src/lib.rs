use std::{collections::HashMap, path::PathBuf};

pub mod rust;
pub mod typescript;

pub struct Job {
    pub name: String,
    pub protocol: tx3_lang::Protocol,
    pub dest_path: PathBuf,
    pub trp_endpoint: String,
    pub trp_headers: HashMap<String, String>,
    pub env_args: HashMap<String, String>,
}

/// Builds the Rust bindings for the given tx3 file
///
/// This is an ergonomic entry-point that is meant to be use from build.rs files
/// of dependent crates.
pub fn build(path: &str) {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", path);

    let path = std::path::Path::new(path);

    let protocol = tx3_lang::Protocol::from_file(path).load().unwrap();

    let out_dir = std::env::var("OUT_DIR").unwrap();

    let job = Job {
        name: path.file_stem().unwrap().to_str().unwrap().to_string(),
        protocol,
        dest_path: out_dir.into(),
        trp_endpoint: "http://localhost:3000".to_string(),
        trp_headers: HashMap::new(),
        env_args: HashMap::new(),
    };

    rust::generate(&job);
}
