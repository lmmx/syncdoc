#![doc = include_str!("../docs/lib.md")]

use syncdoc::omnidoc;

#[allow(missing_docs)] // enum is not supported by syncdoc/omnidoc!
#[omnidoc(path = "../docs")]
pub enum TimeOfDay {
    Day,
    Night,
}
