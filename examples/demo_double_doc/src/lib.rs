#![doc = include_str!("../docs/lib.md")]

use syncdoc::omnidoc;

/// The Settings struct is already documented by a docstring.
///
/// It is joined without a newline: to get a new paragraph for the docstring you would put an extra
/// newline at the end of the markdown doc.
///
/// Even if it is put before the attribute macro, the docstring will be inserted after.
#[derive(Debug)]
#[omnidoc(path = "docs")]
pub struct Settings {
    pub name: String,
    pub switch: bool,
}
