/// Emit a developer-only diagnostic from callback-reachable code.
///
/// In release builds the macro body is removed by `cfg`, including argument
/// evaluation and formatting. Debug builds intentionally trade RT timing
/// fidelity for ordinary functional observability. A runtime level check keeps
/// arguments unevaluated when debug logging is disabled by the installed logger.
#[macro_export]
macro_rules! rt_debug_log {
    ($($arg:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            if $crate::__rt_debug_enabled() {
                $crate::__emit_rt_debug(format_args!($($arg)*));
            }
        }
    }};
}

#[doc(hidden)]
#[cfg(debug_assertions)]
pub fn enabled() -> bool {
    log::log_enabled!(log::Level::Debug)
}

#[doc(hidden)]
#[cfg(debug_assertions)]
pub fn emit(args: std::fmt::Arguments<'_>) {
    log::debug!("{args}");
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
            metadata.level() <= log::Level::Debug
        }

        fn log(&self, _record: &log::Record<'_>) {}

        fn flush(&self) {}
    }

    #[cfg(debug_assertions)]
    fn enable_debug_logging() {
        static LOGGER: DebugTestLogger = DebugTestLogger;
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            log::set_logger(&LOGGER).expect("install test logger");
            log::set_max_level(log::LevelFilter::Debug);
        });
    }

    #[test]
    fn argument_evaluation_matches_diagnostic_build_mode() {
        #[cfg(debug_assertions)]
        enable_debug_logging();
        let evaluations = Cell::new(0);

        crate::rt_debug_log!("value={}", {
            evaluations.set(evaluations.get() + 1);
            42
        });

        assert_eq!(evaluations.get(), usize::from(cfg!(debug_assertions)));
    }
}
