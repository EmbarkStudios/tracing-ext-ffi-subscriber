fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let target_file = match std::env::var("TRACING_FFI_RELATIVE_OUT_PATH") {
        Ok(relative) => std::path::PathBuf::from(&crate_dir).join(relative),
        Err(_) => {
            let target_dir = std::env::var("OUT_DIR").unwrap();
            std::path::PathBuf::from(target_dir).join("../../../../tracing_ffi.h")
        }
    };

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(target_file);
}
