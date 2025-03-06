pub mod rust;
pub mod typescript;

/// Builds the Rust bindings for the given tx3 file
///
/// This is an ergonomic entry-point that is meant to be use from build.rs files
/// of dependent crates.
pub fn build(path: &str) {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", path);

    let protocol = tx3_lang::Protocol::from_file(path).load().unwrap();

    // Create output file
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let filename = path.replace(".tx3", ".rs");
    let dest_path = std::path::Path::new(&out_dir).join(filename);

    rust::generate(protocol, &dest_path);
}
