fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_dir = std::env::var("OUT_DIR").unwrap();
    let target_file = std::path::PathBuf::from(target_dir).join("../../../../libtracing_ffi.h");

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(target_file);
}
