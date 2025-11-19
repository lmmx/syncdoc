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

/// Automatically enable debug output for all tests
#[ctor::ctor]
fn init_debug() {
    syncdoc_core::debug::set_debug(true);
}
