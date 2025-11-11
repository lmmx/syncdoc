#![doc = include_str!("../docs/lib.md")]
#![cfg_attr(doc, deny(missing_docs))]

#[cfg_attr(doc, syncdoc::omnidoc)]
pub enum TimeOfDay {
    Day,
    Night,
}
