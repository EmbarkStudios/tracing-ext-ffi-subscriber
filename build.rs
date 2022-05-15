use std::io::ErrorKind;

fn main() {
    let mut delete_path = None;

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let target_file = match std::env::var("TRACING_FFI_RELATIVE_OUT_PATH") {
        Ok(relative) => std::path::PathBuf::from(&crate_dir).join(relative),
        Err(_) => {
            let target_dir = std::env::var("OUT_DIR").unwrap();
            delete_path = Some(std::path::PathBuf::from(&crate_dir).join("Cargo.lock"));
            std::path::PathBuf::from(target_dir).join("tracing_ffi.h")
        }
    };

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(target_file);

    if let Some(dummy_manifest) = delete_path {
        if let Err(e) = std::fs::remove_file(&dummy_manifest) {
            if e.kind() != ErrorKind::NotFound {
                panic!("failed deleting dummy Cargo.lock: {}", e);
            }
        }
    }
}
