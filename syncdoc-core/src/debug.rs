//! Debug printer control for syncdoc.
//!
//! Provides a thread-safe atomic flag for debug logging via STDERR and a function
//! to enable it programmatically (runs automatically if compiled in `cfg(test)`).

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};

/// Atomic flag indicating whether debug output is enabled.
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Initialise the debug atomic from the `SYNCDOC_DEBUG` environment variable.
///
/// - Treats `"0"`, `"false"`, `"no"`, `"off"` as false.
/// - Any other value is true.
/// - If the variable is unset, defaults to true for tests, false otherwise.
pub fn init_from_env() {
    let enabled = match env::var("SYNCDOC_DEBUG") {
        Ok(val) => {
            let val = val.trim();
            !(val == "0"
                || val.eq_ignore_ascii_case("false")
                || val.eq_ignore_ascii_case("no")
                || val.eq_ignore_ascii_case("off"))
        }
        Err(_) => cfg!(test),
    };
    set_debug(enabled);
}

/// Enable or disable debug output programmatically.
pub fn set_debug(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check whether debug output is enabled.
pub fn is_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

/// Automatically enable debug output for tests, respecting the env var.
#[ctor::ctor]
fn init_debug() {
    init_from_env();
}
