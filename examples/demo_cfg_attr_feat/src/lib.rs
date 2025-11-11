#![doc = include_str!("../docs/lib.md")]
#![cfg_attr(doc, deny(missing_docs))]

#[syncdoc::omnidoc]
pub enum TimeOfDay {
    Day,
    Night,
}
