#![doc = include_str!("../docs/lib.md")]
#![cfg_attr(doc, deny(missing_docs))]

#[cfg(doc)]
use syncdoc::omnidoc;

#[omnidoc]
pub enum TimeOfDay {
    Day,
    Night,
}
