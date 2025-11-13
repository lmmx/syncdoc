#![doc = include_str!("../docs/lib.md")]

use syncdoc::omnidoc;

#[omnidoc(path = "docs")]
pub enum TimeOfDay {
    Day,
    Night,
}
