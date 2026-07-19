/// Internal implementation for developer-only callback diagnostics.
///
/// In release builds the body is removed by `cfg`, including level/argument
/// evaluation and formatting. Debug builds intentionally trade RT timing
/// fidelity for ordinary functional observability.
#[doc(hidden)]
#[macro_export]
macro_rules! __rt_log {
    ($level:expr, $($arg:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            let level = $level;
            if $crate::__rt_log_enabled(level) {
                $crate::__emit_rt_log(level, format_args!($($arg)*));
            }
        }
    }};
}

#[macro_export]
macro_rules! rt_debug_log {
    ($($arg:tt)*) => {{
        $crate::__rt_log!($crate::__RtLogLevel::Debug, $($arg)*)
    }};
}

#[macro_export]
macro_rules! rt_info_log {
    ($($arg:tt)*) => {{
        $crate::__rt_log!($crate::__RtLogLevel::Info, $($arg)*)
    }};
}

#[macro_export]
macro_rules! rt_warn_log {
    ($($arg:tt)*) => {{
        $crate::__rt_log!($crate::__RtLogLevel::Warn, $($arg)*)
    }};
}

#[macro_export]
macro_rules! rt_error_log {
    ($($arg:tt)*) => {{
        $crate::__rt_log!($crate::__RtLogLevel::Error, $($arg)*)
    }};
}

#[doc(hidden)]
#[cfg(debug_assertions)]
pub fn enabled(level: log::Level) -> bool {
    log::log_enabled!(level)
}

#[doc(hidden)]
#[cfg(debug_assertions)]
pub fn emit(level: log::Level, args: std::fmt::Arguments<'_>) {
    log::log!(level, "{args}");
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    #[cfg(debug_assertions)]
    use std::sync::Once;

    #[cfg(debug_assertions)]
    struct DebugTestLogger;

    #[cfg(debug_assertions)]
    impl log::Log for DebugTestLogger {
        fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
            metadata.level() <= log::max_level()
        }

        fn log(&self, _record: &log::Record<'_>) {}

        fn flush(&self) {}
    }

    #[cfg(debug_assertions)]
    fn enable_debug_logging() {
        static LOGGER: DebugTestLogger = DebugTestLogger;
        static INIT: Once = Once::new();
        INIT.call_once(|| log::set_logger(&LOGGER).expect("install test logger"));
        log::set_max_level(log::LevelFilter::Trace);
    }

    #[test]
    fn argument_evaluation_matches_build_mode_and_runtime_level() {
        #[cfg(debug_assertions)]
        enable_debug_logging();
        let evaluations = Cell::new(0);
        let _value = || {
            evaluations.set(evaluations.get() + 1);
            42
        };

        crate::rt_debug_log!("debug={}", _value());
        crate::rt_info_log!("info={}", _value());
        crate::rt_warn_log!("warn={}", _value());
        crate::rt_error_log!("error={}", _value());

        #[cfg(debug_assertions)]
        log::set_max_level(log::LevelFilter::Error);
        crate::rt_debug_log!("disabled debug={}", _value());
        crate::rt_error_log!("enabled error={}", _value());

        assert_eq!(
            evaluations.get(),
            if cfg!(debug_assertions) { 5 } else { 0 }
        );
    }
}
