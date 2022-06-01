use std::io::ErrorKind;

use cbindgen::Language;

fn main() {
    if std::env::var_os("DOCS_RS").is_some() {
        return;
    }

    let mut delete_path = None;
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let crate_dir = std::path::PathBuf::from(&crate_dir);
    let config = cbindgen::Config::from_file(crate_dir.join("cbindgen.toml")).unwrap();
    let mut c_config = config.clone();
    c_config.language = Language::C;
    let target_dir = std::env::var("OUT_DIR").unwrap();

    let target_dir = match std::env::var("TRACING_FFI_RELATIVE_OUT_DIR") {
        Ok(relative) => std::path::PathBuf::from(&target_dir).join(relative),
        Err(_) => {
            delete_path = Some(crate_dir.join("Cargo.lock"));
            std::path::PathBuf::from(target_dir)
        }
    };

    cbindgen::generate_with_config(&crate_dir, config)
        .expect("Unable to generate bindings")
        .write_to_file(target_dir.join("tracing_ffi.hpp"));

    cbindgen::generate_with_config(&crate_dir, c_config)
        .expect("Unable to generate bindings")
        .write_to_file(target_dir.join("tracing_ffi.h"));

    if let Some(dummy_manifest) = delete_path {
        if let Err(e) = std::fs::remove_file(&dummy_manifest) {
            if e.kind() != ErrorKind::NotFound {
                panic!("failed deleting dummy Cargo.lock: {}", e);
            }
        }
    }
}
