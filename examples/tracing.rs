use std::ffi::CStr;

use tracing_ext_ffi_subscriber::Configuration;

unsafe extern "C" fn enter_fn(_: *const i8) {}
unsafe extern "C" fn exit_fn(_: *const i8) {}
unsafe extern "C" fn enabled_fn() -> bool {
    true
}

unsafe extern "C" fn on_event_fn(level: tracing_ext_ffi_subscriber::LogLevel, msg: *const i8) {
    eprintln!("{:?}: {}", level, CStr::from_ptr(msg).to_string_lossy());
}

fn main() {
    let config = Configuration {
        enter_fn: Some(enter_fn),
        exit_fn: Some(exit_fn),
        enabled_fn: Some(enabled_fn),
        on_event_fn: Some(on_event_fn),
        event_enabled_fn: None,
    };

    unsafe { tracing_ext_ffi_subscriber::tracing_ffi_install_global_with_config(config) };

    tracing::info!(test = "123", "foobar");
    tracing::debug!("foobar");
    tracing::trace!("foobar");
}
