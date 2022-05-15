use std::os::raw::c_char;
pub mod subscriber;
/// Function to be called when entering a tracing scope.

///
/// Safety: The pointee of name is not guaranteed to live after the call finishes.
pub type StartTraceScopeFn = unsafe extern "C" fn(name: *const c_char);

/// Function to be called when exiting a tracing scope.
///
/// Safety: The pointee of name is not guaranteed to live after the call finishes.
pub type EndTraceScopeFn = unsafe extern "C" fn(name: *const c_char);

/// Function to call to check whether tracing is enabled.
pub type IsEnabledFn = unsafe extern "C" fn() -> bool;

/// Simple error codes used for FFI calls.
#[repr(C)]
pub enum ReturnCode {
    Success = 0,
    Failure = 1,
}

/// Install the tracing hook globally with the provided enter and exit functions.
///
/// Safety: The function pointers must be valid functions matching the provided signature.
#[no_mangle]
pub unsafe extern "C" fn tracing_ffi_install_global(
    enter_fn: StartTraceScopeFn,
    exit_fn: EndTraceScopeFn,
) -> ReturnCode {
    let subscriber = subscriber::ExternFFISpanSubscriber::new(enter_fn, exit_fn);
    match tracing::subscriber::set_global_default(subscriber) {
        Ok(_) => ReturnCode::Success,
        Err(_) => ReturnCode::Failure,
    }
}

/// Install the tracing hook globally with the provided enter, exit, and enabled functions.
///
/// Safety: The function pointers must be valid functions matching the provided signature.
#[no_mangle]
pub unsafe extern "C" fn tracing_ffi_install_global_with_enabled(
    enter_fn: StartTraceScopeFn,
    exit_fn: EndTraceScopeFn,
    enabled_fn: IsEnabledFn,
) -> ReturnCode {
    let subscriber =
        subscriber::ExternFFISpanSubscriber::new_with_enabled(enter_fn, exit_fn, enabled_fn);
    match tracing::subscriber::set_global_default(subscriber) {
        Ok(_) => ReturnCode::Success,
        Err(_) => ReturnCode::Failure,
    }
}
