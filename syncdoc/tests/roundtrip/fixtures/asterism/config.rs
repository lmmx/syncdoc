//! Configuration to acknowledge developer preferences as well as set defaults.
//!
//! Specifically, we try to find an asterism.toml, and if present we load settings from there.
//! This provides wrapping width and file extension preferences.

use facet::Facet;
use std::fs;

/// User preferences loaded from asterism.toml or falling back to defaults.
#[derive(Facet, Clone)]
pub struct Config {
    /// Maximum line width for editor text wrapping.
    #[facet(default = 100)]
    pub wrap_width: usize,
    #[facet(default = vec!["md".to_string()])]
    /// File suffixes to match when scanning directories.
    pub file_extensions: Vec<String>,
}

impl Config {
    /// Load configuration from asterism.toml if present.
    ///
    /// # Panics
    ///
    /// Panics if the default configuration cannot be parsed.
    #[must_use]
    pub fn load() -> Self {
        if let Ok(contents) = fs::read_to_string("asterism.toml") {
            if let Ok(config) = facet_toml::from_str::<Self>(&contents) {
                return config;
            }
        }
        facet_toml::from_str::<Self>("").unwrap()
    }
}
