//! Debug printer control for syncdoc.
//!
//! Provides a thread-safe atomic flag for debug logging via STDERR and a function
//! to enable it programmatically (used in tests).

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};

/// Atomic flag indicating whether debug output is enabled.
/// Initialised at runtime from `SYNCDOC_DEBUG`.
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Initialise the debug atomic from the environment variable.
/// Typically called at program start, or via ctor in tests.
pub fn init_from_env() {
    if env::var("SYNCDOC_DEBUG").is_ok() {
        DEBUG_ENABLED.store(true, Ordering::Relaxed);
    }
}

/// Enable or disable debug output programmatically.
/// Tests can call this directly, or you can wire it via a ctor.
pub fn set_debug(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check whether debug output is enabled.
pub fn is_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}
