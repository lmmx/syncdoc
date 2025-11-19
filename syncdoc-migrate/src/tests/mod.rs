mod bookend;
mod diff;
mod discover;
mod expected;
mod extract;
mod inject;
mod reformat;
mod restore;
mod rewrite;
mod strip;
mod write;

use crate::debug;

/// Automatically enable debug output for all tests
#[ctor::ctor]
fn init_debug() {
    debug::set_debug(true);
}
