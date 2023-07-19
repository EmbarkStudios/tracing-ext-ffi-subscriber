/*!
The subscriber implementation provided by tracing-ext-ffi-subscriber.
*/
use std::fmt;
use std::io::Write;

use std::{collections::HashMap, ffi::CString, os::raw::c_char};
use tracing::field::{Field, Visit};
use tracing::{span, Level};

use crate::{
    EndTraceScopeFn, IsEnabledFn, IsEventEnabledFn, LogLevel, OnEventFn, StartTraceScopeFn,
};

/// A subscriber used for forwarding span enter/exit events to C or C++ code.
pub struct ExternFFISpanSubscriber {
    counter: std::sync::atomic::AtomicU64,
    labels: parking_lot::RwLock<HashMap<u64, CString>>,

    enter_fn: Option<StartTraceScopeFn>,
    exit_fn: Option<EndTraceScopeFn>,
    enabled_fn: Option<IsEnabledFn>,

    on_event_fn: Option<OnEventFn>,
    is_event_enabled_fn: Option<IsEventEnabledFn>,
}

impl ExternFFISpanSubscriber {
    /// Create a new subscriber with the provided callback functions.
    pub fn new(enter_fn: StartTraceScopeFn, exit_fn: EndTraceScopeFn) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(1),
            labels: Default::default(),

            enter_fn: Some(enter_fn),
            exit_fn: Some(exit_fn),
            enabled_fn: None,

            on_event_fn: None,
            is_event_enabled_fn: None,
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

            enter_fn: Some(enter_fn),
            exit_fn: Some(exit_fn),
            enabled_fn: Some(enabled_fn),
            on_event_fn: None,
            is_event_enabled_fn: None,
        }
    }

    pub fn new_generic(
        enter_fn: Option<StartTraceScopeFn>,
        exit_fn: Option<EndTraceScopeFn>,
        enabled_fn: Option<IsEnabledFn>,
        on_event_fn: Option<OnEventFn>,
        is_event_enabled_fn: Option<IsEventEnabledFn>,
    ) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(1),
            labels: Default::default(),

            enter_fn,
            exit_fn,
            enabled_fn,
            on_event_fn,
            is_event_enabled_fn,
        }
    }
}

// Derived from `tracing-subscriber` setup for compact formatting.
struct Visitor<'a> {
    writer: &'a mut dyn Write,
    is_empty: bool,
}

impl<'a> Visitor<'a> {
    pub fn new(writer: &'a mut dyn Write, is_empty: bool) -> Self {
        Self { writer, is_empty }
    }

    fn maybe_pad(&mut self) {
        if self.is_empty {
            self.is_empty = false;
        } else {
            write!(self.writer, " ").unwrap();
        }
    }
}

impl<'a> Visit for Visitor<'a> {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.record_debug(field, &format_args!("{}", value))
        } else {
            self.record_debug(field, &value)
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        if let Some(source) = value.source() {
            self.record_debug(
                field,
                &format_args!("{} {} = {}", value, field.name(), source,),
            )
        } else {
            self.record_debug(field, &format_args!("{}", value))
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.maybe_pad();
        match field.name() {
            "message" => write!(self.writer, "{:?}", value),
            name if name.starts_with("log.") => Ok(()),
            name if name.starts_with("r#") => write!(self.writer, "{}={:?}", &name[2..], value),
            name => write!(self.writer, "{}={:?}", name, value),
        }
        .unwrap();
    }
}

impl tracing::Subscriber for ExternFFISpanSubscriber {
    /// Intern the span as a C pointer.
    fn new_span(&self, attrs: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        let id = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let name = match CString::new(attrs.metadata().name()) {
            Ok(n) => n,
            Err(_) => CString::new("malformed name").unwrap(),
        };

        self.labels.write().insert(id, name);
        tracing::span::Id::from_u64(id)
    }

    /// Record entering the span.
    fn enter(&self, id: &tracing::span::Id) {
        if let Some(enter_fn) = self.enter_fn {
            let labels = self.labels.read();
            let name = labels.get(&id.into_u64()).unwrap();

            unsafe { (enter_fn)(name.as_ptr() as *const c_char) }
        }
    }

    /// Record exiting the span.
    fn exit(&self, id: &tracing::span::Id) {
        if let Some(exit_fn) = self.exit_fn {
            let labels = self.labels.read();
            let name = labels.get(&id.into_u64()).unwrap();
            unsafe { (exit_fn)(name.as_ptr() as *const c_char) }
        }
    }

    /// Will either check the provided `enabled_fn` passed to [`Self::new_with_enabled`] or true.
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        if metadata.is_span() {
            self.enabled_fn.map(|f| unsafe { (f)() }).unwrap_or(true)
        } else if metadata.is_event() {
            let log_level = match *metadata.level() {
                Level::TRACE => LogLevel::Trace,
                Level::DEBUG => LogLevel::Debug,
                Level::INFO => LogLevel::Info,
                Level::WARN => LogLevel::Warn,
                Level::ERROR => LogLevel::Error,
            };

            self.is_event_enabled_fn
                .map_or(true, |f| unsafe { (f)(log_level) })
        } else {
            false
        }
    }

    /// Formats and forwards a message to the host with the provided level.
    fn event(&self, event: &tracing::Event<'_>) {
        let Some(log_fn) = self.on_event_fn else {
            return;
        };

        let level = event.metadata().level();
        let log_level = match *level {
            Level::TRACE => LogLevel::Trace,
            Level::DEBUG => LogLevel::Debug,
            Level::INFO => LogLevel::Info,
            Level::WARN => LogLevel::Warn,
            Level::ERROR => LogLevel::Error,
        };

        let should_log = self
            .is_event_enabled_fn
            .map_or(true, |f| unsafe { f(log_level) });

        if !should_log {
            return;
        }

        let mut bytes: Vec<u8> = vec![];
        let mut visitor = Visitor::new(&mut bytes, true);

        event.record(&mut visitor);

        let name = CString::new(bytes).unwrap();

        unsafe { log_fn(log_level, name.as_ptr()) }
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
}
