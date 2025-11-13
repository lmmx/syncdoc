#![doc = " A lib.rs module containing one public enum."]
#[syncdoc::omnidoc(path = "docs")]
pub enum TimeOfDay {
    Day,
    Night,
}
