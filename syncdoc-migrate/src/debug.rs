use std::env;
use std::sync::atomic::{AtomicBool, Ordering};

/// Atomic flag indicating whether debug output is enabled.
/// Initialised once from the `SYNCDOC_DEBUG` environment variable.
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(env::var("SYNCDOC_DEBUG").is_ok());

/// Enable or disable debug output programmatically.
///
/// Useful in tests or runtime scenarios where you want to toggle debug
/// output without using environment variables.
pub fn set_debug(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Query whether debug output is currently enabled.
///
/// Returns `true` if debugging is enabled, `false` otherwise.
pub fn is_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}
