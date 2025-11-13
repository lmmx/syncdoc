#![doc = include_str!("../docs/lib.md")]

use syncdoc::omnidoc;

#[omnidoc(path = "docs")]
pub fn hello(who: &str) -> String {
    format!("Hello {}", who)
}
