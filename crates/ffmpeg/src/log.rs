use std::ffi::CStr;
use std::ptr::NonNull;
use std::sync::Arc;

use arc_swap::ArcSwapOption;
use nutype_enum::nutype_enum;

use crate::ffi::*;

nutype_enum! {
    /// The logging level
    pub enum LogLevel(i32) {
        /// Quiet logging level.
        Quiet = AV_LOG_QUIET,
        /// Panic logging level.
        Panic = AV_LOG_PANIC as i32,
        /// Fatal logging level.
        Fatal = AV_LOG_FATAL as i32,
        /// Error logging level.
        Error = AV_LOG_ERROR as i32,
        /// Warning logging level.
        Warning = AV_LOG_WARNING as i32,
        /// Info logging level.
        Info = AV_LOG_INFO as i32,
        /// Verbose logging level.
        Verbose = AV_LOG_VERBOSE as i32,
        /// Debug logging level.
        Debug = AV_LOG_DEBUG as i32,
        /// Trace logging level.
        Trace = AV_LOG_TRACE as i32,
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Quiet => write!(f, "quiet"),
            Self::Panic => write!(f, "panic"),
            Self::Fatal => write!(f, "fatal"),
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Info => write!(f, "info"),
            Self::Verbose => write!(f, "verbose"),
            Self::Debug => write!(f, "debug"),
            Self::Trace => write!(f, "trace"),
            Self(int) => write!(f, "unknown({int})"),
        }
    }
}

/// Sets the log level.
pub fn set_log_level(level: LogLevel) {
    // Safety: `av_log_set_level` is safe to call.
    unsafe {
        av_log_set_level(level.0);
    }
}

type Function = Box<dyn Fn(LogLevel, Option<String>, String) + Send + Sync>;
static LOG_CALLBACK: ArcSwapOption<Function> = ArcSwapOption::const_empty();

/// Sets the log callback.
#[inline(always)]
pub fn log_callback_set(callback: impl Fn(LogLevel, Option<String>, String) + Send + Sync + 'static) {
    log_callback_set_boxed(Box::new(callback));
}

/// Sets the log callback.
pub fn log_callback_set_boxed(callback: Function) {
    LOG_CALLBACK.store(Some(Arc::new(callback)));

    // Safety: the `log_cb` function has the same structure as the required `AVLogCallback` function.
    // The reason we do this transmute is because of the way `VaList` is defined on different architectures.
    #[allow(clippy::missing_transmute_annotations)]
    let log_cb_transmuted = unsafe { std::mem::transmute(log_cb as *const ()) };
    // Safety: `av_log_set_callback` is safe to call.
    unsafe {
        av_log_set_callback(Some(log_cb_transmuted));
    }
}

/// Unsets the log callback.
pub fn log_callback_unset() {
    LOG_CALLBACK.store(None);

    // Safety: `av_log_set_callback` is safe to call.
    unsafe {
        av_log_set_callback(None);
    }
}

unsafe extern "C" {
    fn vsnprintf(
        str: *mut libc::c_char,
        size: libc::size_t,
        format: *const libc::c_char,
        ap: ::va_list::VaList,
    ) -> libc::c_int;
}

unsafe extern "C" fn log_cb(ptr: *mut libc::c_void, level: libc::c_int, fmt: *const libc::c_char, va: ::va_list::VaList) {
    let guard = LOG_CALLBACK.load();
    let Some(cb) = guard.as_ref() else {
        return;
    };

    let level = LogLevel::from(level);
    let class = NonNull::new(ptr as *mut *mut AVClass)
        .and_then(|class| {
            // Safety: The pointer is valid
            NonNull::new(unsafe { *class.as_ptr() })
        })
        .and_then(|class| {
            // Safety: The pointer is valid
            let class = unsafe { class.as_ref() };
            let im = class.item_name?;
            // Safety: The pointer is valid
            let c_str = unsafe { im(ptr) };
            // Safety: The returned pointer is a valid CString
            let c_str = unsafe { CStr::from_ptr(c_str as *const _) };

            Some(c_str.to_string_lossy().trim().to_owned())
        });

    let mut buf: [std::os::raw::c_char; 1024] = [0; 1024];

    // Safety: The pointer is valid and the buffer has enough bytes with the max length set.
    unsafe {
        vsnprintf(buf.as_mut_ptr() as *mut _, buf.len() as _, fmt, va);
    }

    // Safety: The pointer is valid and the buffer has enough bytes with the max length set.
    let c_str = unsafe { CStr::from_ptr(buf.as_ptr() as *const _) };
    let msg = c_str.to_string_lossy().trim().to_owned();

    cb(level, class, msg);
}

/// Sets the log callback to use tracing.
#[cfg(feature = "tracing")]
#[cfg_attr(docsrs, doc(cfg(feature = "tracing")))]
pub fn log_callback_tracing() {
    log_callback_set(|mut level, class, msg| {
        let class = class.as_deref().unwrap_or("ffmpeg");

        // We purposely ignore this message because it's a false positive
        if msg == "deprecated pixel format used, make sure you did set range correctly" {
            level = LogLevel::Debug;
        }

        match level {
            LogLevel::Trace => tracing::trace!("{level}: {class} @ {msg}"),
            LogLevel::Verbose => tracing::trace!("{level}: {class} @ {msg}"),
            LogLevel::Debug => tracing::debug!("{level}: {class} @ {msg}"),
            LogLevel::Info => tracing::info!("{level}: {class} @ {msg}"),
            LogLevel::Warning => tracing::warn!("{level}: {class} @ {msg}"),
            LogLevel::Quiet => tracing::error!("{level}: {class} @ {msg}"),
            LogLevel::Error => tracing::error!("{level}: {class} @ {msg}"),
            LogLevel::Panic => tracing::error!("{level}: {class} @ {msg}"),
            LogLevel::Fatal => tracing::error!("{level}: {class} @ {msg}"),
            LogLevel(_) => tracing::debug!("{level}: {class} @ {msg}"),
        }
    });
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::ffi::CString;
    use std::sync::{Arc, Mutex};

    use crate::AVCodecID;
    use crate::ffi::{av_log, av_log_get_level, avcodec_find_decoder};
    use crate::log::{LogLevel, log_callback_set, log_callback_unset, set_log_level};

    #[test]
    fn test_log_level_as_str_using_from_i32() {
        let test_cases = [
            (LogLevel::Quiet, "quiet"),
            (LogLevel::Panic, "panic"),
            (LogLevel::Fatal, "fatal"),
            (LogLevel::Error, "error"),
            (LogLevel::Warning, "warning"),
            (LogLevel::Info, "info"),
            (LogLevel::Verbose, "verbose"),
            (LogLevel::Debug, "debug"),
            (LogLevel::Trace, "trace"),
            (LogLevel(100), "unknown(100)"),
            (LogLevel(-1), "unknown(-1)"),
        ];

        for &(input, expected) in &test_cases {
            let log_level = input;
            assert_eq!(
                log_level.to_string(),
                expected,
                "Expected '{expected}' for input {input}, but got '{log_level}'"
            );
        }
    }

    #[test]
    fn test_set_log_level() {
        let log_levels = [
            LogLevel::Quiet,
            LogLevel::Panic,
            LogLevel::Fatal,
            LogLevel::Error,
            LogLevel::Warning,
            LogLevel::Info,
            LogLevel::Verbose,
            LogLevel::Debug,
            LogLevel::Trace,
        ];

        for &level in &log_levels {
            set_log_level(level);
            // Safety: `av_log_get_level` is safe to call.
            let current_level = unsafe { av_log_get_level() };

            assert_eq!(
                current_level, level.0,
                "Expected log level to be {}, but got {}",
                level.0, current_level
            );
        }
    }

    #[test]
    fn test_log_callback_set() {
        let captured_logs = Arc::new(Mutex::new(Vec::new()));
        let callback_logs = Arc::clone(&captured_logs);
        log_callback_set(move |level, class, message| {
            let mut logs = callback_logs.lock().unwrap();
            logs.push((level, class, message));
        });

        let log_message = CString::new("Test warning log message").expect("Failed to create CString");
        // Safety: `av_log` is safe to call.
        unsafe {
            av_log(std::ptr::null_mut(), LogLevel::Warning.0, log_message.as_ptr());
        }

        let logs = captured_logs.lock().unwrap();
        assert_eq!(logs.len(), 1, "Expected one log message to be captured");

        let (level, class, message) = &logs[0];
        assert_eq!(*level, LogLevel::Warning, "Expected log level to be Warning");
        assert!(class.is_none(), "Expected class to be None for this test");
        assert_eq!(message, "Test warning log message", "Expected log message to match");
        log_callback_unset();
    }

    #[test]
    fn test_log_callback_with_class() {
        // Safety: `avcodec_find_decoder` is safe to call.
        let codec = unsafe { avcodec_find_decoder(AVCodecID::H264.into()) };
        assert!(!codec.is_null(), "Failed to find H264 codec");

        // Safety: `(*codec).priv_class` is safe to access.
        let av_class_ptr = unsafe { (*codec).priv_class };
        assert!(!av_class_ptr.is_null(), "AVClass for codec is null");

        let captured_logs = Arc::new(Mutex::new(Vec::new()));

        let callback_logs = Arc::clone(&captured_logs);
        log_callback_set(move |level, class, message| {
            let mut logs = callback_logs.lock().unwrap();
            logs.push((level, class, message));
        });

        // Safety: `av_log` is safe to call.
        unsafe {
            av_log(
                &av_class_ptr as *const _ as *mut _,
                LogLevel::Info.0,
                CString::new("Test log message with real AVClass").unwrap().as_ptr(),
            );
        }

        let logs = captured_logs.lock().unwrap();
        assert_eq!(logs.len(), 1, "Expected one log message to be captured");

        let (level, class, message) = &logs[0];
        assert_eq!(*level, LogLevel::Info, "Expected log level to be Info");
        assert!(class.is_some(), "Expected class name to be captured");
        assert_eq!(message, "Test log message with real AVClass", "Expected log message to match");
        log_callback_unset();
    }

    #[test]
    fn test_log_callback_unset() {
        let captured_logs = Arc::new(Mutex::new(Vec::new()));
        let callback_logs = Arc::clone(&captured_logs);
        log_callback_set(move |level, class, message| {
            let mut logs = callback_logs.lock().unwrap();
            logs.push((level, class, message));
        });

        // Safety: `av_log` is safe to call.
        unsafe {
            av_log(
                std::ptr::null_mut(),
                LogLevel::Info.0,
                CString::new("Test log message before unset").unwrap().as_ptr(),
            );
        }

        {
            let logs = captured_logs.lock().unwrap();
            assert_eq!(
                logs.len(),
                1,
                "Expected one log message to be captured before unsetting the callback"
            );
            let (_, _, message) = &logs[0];
            assert_eq!(message, "Test log message before unset", "Expected the log message to match");
        }

        log_callback_unset();

        // Safety: `av_log` is safe to call.
        unsafe {
            av_log(
                std::ptr::null_mut(),
                LogLevel::Info.0,
                CString::new("Test log message after unset").unwrap().as_ptr(),
            );
        }

        let logs = captured_logs.lock().unwrap();
        assert_eq!(
            logs.len(),
            1,
            "Expected no additional log messages to be captured after unsetting the callback"
        );
    }

    #[cfg(feature = "tracing")]
    #[test]
    #[tracing_test::traced_test]
    fn test_log_callback_tracing() {
        use tracing::Level;
        use tracing::subscriber::set_default;
        use tracing_subscriber::FmtSubscriber;

        use crate::log::log_callback_tracing;

        let subscriber = FmtSubscriber::builder().with_max_level(Level::TRACE).finish();
        // Intentional improper error handling to cause an error that we handle later in the test.
        let _ = set_default(subscriber);
        log_callback_tracing();

        let levels_and_expected_tracing = [
            (LogLevel::Trace, "trace"),
            (LogLevel::Verbose, "trace"),
            (LogLevel::Debug, "debug"),
            (LogLevel::Info, "info"),
            (LogLevel::Warning, "warning"),
            (LogLevel::Quiet, "error"),
            (LogLevel::Error, "error"),
            (LogLevel::Panic, "error"),
            (LogLevel::Fatal, "error"),
        ];

        for (level, expected_tracing_level) in &levels_and_expected_tracing {
            let message = format!("Test {expected_tracing_level} log message");
            // Safety: `av_log` is safe to call.
            unsafe {
                av_log(
                    std::ptr::null_mut(),
                    level.0,
                    CString::new(message.clone()).expect("Failed to create CString").as_ptr(),
                );
            }
        }

        for (_level, expected_tracing_level) in &levels_and_expected_tracing {
            let expected_message = format!("{expected_tracing_level}: ffmpeg @ Test {expected_tracing_level} log message");

            assert!(
                logs_contain(&expected_message),
                "Expected log message for '{expected_message}'"
            );
        }
        log_callback_unset();
    }

    #[cfg(feature = "tracing")]
    #[test]
    #[tracing_test::traced_test]
    fn test_log_callback_tracing_deprecated_message() {
        use tracing::Level;
        use tracing::subscriber::set_default;
        use tracing_subscriber::FmtSubscriber;

        use crate::log::log_callback_tracing;

        let subscriber = FmtSubscriber::builder().with_max_level(Level::TRACE).finish();
        // Intentional improper error handling to cause an error that we handle later in the test.
        let _ = set_default(subscriber);
        log_callback_tracing();

        let deprecated_message = "deprecated pixel format used, make sure you did set range correctly";
        // Safety: `av_log` is safe to call.
        unsafe {
            av_log(
                std::ptr::null_mut(),
                LogLevel::Trace.0,
                CString::new(deprecated_message).expect("Failed to create CString").as_ptr(),
            );
        }

        assert!(
            logs_contain(&format!("debug: ffmpeg @ {deprecated_message}")),
            "Expected log message for '{deprecated_message}'"
        );
        log_callback_unset();
    }
}
