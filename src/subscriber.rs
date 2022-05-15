/*!
The subscriber implementation provided by tracing-ext-ffi-subscriber.
*/

use std::{collections::HashMap, ffi::CString, os::raw::c_char};
use tracing::span;

use crate::{EndTraceScopeFn, IsEnabledFn, StartTraceScopeFn};

/// A subscriber used for forwarding span enter/exit events to C or C++ code.
pub struct ExternFFISpanSubscriber {
    counter: std::sync::atomic::AtomicU64,
    labels: parking_lot::RwLock<HashMap<u64, CString>>,

    enter_fn: StartTraceScopeFn,
    exit_fn: EndTraceScopeFn,
    enabled_fn: Option<IsEnabledFn>,
}

impl ExternFFISpanSubscriber {
    /// Create a new subscriber with the provided callback functions.
    pub fn new(enter_fn: StartTraceScopeFn, exit_fn: EndTraceScopeFn) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(1),
            labels: Default::default(),

            enter_fn,
            exit_fn,
            enabled_fn: None,
        }
    }

    /// Create a new subscriber with the provided callback functions.
    pub fn new_with_enabled(
        enter_fn: StartTraceScopeFn,
        exit_fn: EndTraceScopeFn,
        enabled_fn: IsEnabledFn,
    ) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(1),
            labels: Default::default(),

            enter_fn,
            exit_fn,
            enabled_fn: Some(enabled_fn),
        }
    }
}

impl tracing::Subscriber for ExternFFISpanSubscriber {
    /// Intern the span as a C pointer.
    fn new_span(&self, attrs: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        let id = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let name = attrs.fields().field("name").unwrap();

        let name = CString::new(name.as_ref()).unwrap_or_else(|_| {
            // Safety: This can never contain internal 0 bytes
            CString::new("malformed_string").unwrap()
        });

        self.labels.write().insert(id, name);
        tracing::span::Id::from_u64(id)
    }

    /// Record entering the span.
    fn enter(&self, id: &tracing::span::Id) {
        let labels = self.labels.read();
        let name = labels.get(&id.into_u64()).unwrap();

        unsafe { (self.enter_fn)(name.as_ptr() as *const c_char) }
    }

    /// Record exiting the span.
    fn exit(&self, id: &tracing::span::Id) {
        let labels = self.labels.read();
        let name = labels.get(&id.into_u64()).unwrap();
        unsafe { (self.exit_fn)(name.as_ptr() as *const c_char) }
    }

    /// Will either check the provided `enabled_fn` passed to [`Self::new_with_enabled`] or true.
    fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
        self.enabled_fn.map(|f| unsafe { (f)() }).unwrap_or(true)
    }

    /// Currently not supported.
    ///
    /// Passing arbitrary values across an FFI boundary becomes increasingly opinionated towards supporting things that
    /// are like tracing.  This isn't explicitly out of scope for this library; but would require some good real-life
    /// examples (i.e., C or C++ host APIs) that would be supported, while having the same FFI API from Rust's
    /// perspective.
    ///
    /// PRs welcome!
    fn record(&self, _: &span::Id, _: &span::Record<'_>) {}

    /// Currently not supported.
    ///
    /// See [`Self::record`](struct.ExternFFISpanSubscriber.html#method.record) for motivation.
    fn record_follows_from(&self, _: &span::Id, _: &span::Id) {}

    /// Currently not supported.
    ///
    /// See [`Self::record`](struct.ExternFFISpanSubscriber.html#method.record) for motivation.
    fn event(&self, _: &tracing::Event<'_>) {}
}
