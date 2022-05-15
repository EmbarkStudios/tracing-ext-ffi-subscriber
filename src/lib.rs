/*!
A simple crate for passing spans generated by the tracing ecosystem to a C or C++ profiling system when Rust is
integrated into an existing framework. To help with integration into other tools you can use the environment variable
`TRACING_FFI_RELATIVE_OUT_PATH` to configure where the include file ends up relative to the Cargo.toml being built.


```c
#include <Profiling.h>
#include <tracing_ffi.h>
#include <myrustlib.h>

int main(int argc, const char* argv[]) {
    tracing_ffi_ReturnCode result = tracing_ffi_install_global_with_enable(
        profiling_begin_named_scope,
        profiling_end_named_scope,
        profiling_is_enabled,
    );

    if (result != tracing_ffi_ReturnCode_Success) {
        return (int)result;
    }

    myrustlib_execute(10, 20);

    profiling_write_file("profile.json");
}
```

You can of course also configure this from Rust code; and bypass the C-api. In that case, use
[`subscriber::ExternFFISpanSubscriber`] directly, and install with your preferred [`tracing`] method.
*/

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
