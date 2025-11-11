#![doc = include_str!("../docs/lib.md")]

use syncdoc::omnidoc;

#[omnidoc]
#[derive(Debug)]
pub struct Settings {
    pub name: String,
    pub switch: bool,
}
