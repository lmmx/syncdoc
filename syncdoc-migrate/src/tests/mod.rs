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
use ctor::ctor;

#[ctor]
fn init_debug() {
    // Enable debug output for all tests automatically
    debug::set_debug(true);
}
