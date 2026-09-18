//! `site.toml`: the values templates see as `site.*`, plus build settings.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub title: String,
    pub email: String,
    pub description: String,
    #[serde(default)]
    pub baseurl: String,
    pub url: String,
    pub twitter_username: Option<String>,
    pub github_username: Option<String>,
    /// Color theme for highlighted code (a two-face embedded theme name).
    #[serde(default = "default_syntax_theme")]
    pub syntax_theme: String,
    /// Root-relative paths that are not part of the site.
    #[serde(default)]
    pub exclude: Vec<String>,
}

fn default_syntax_theme() -> String {
    "GitHub".to_string()
}

impl Config {
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join("site.toml");
        let text = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
    }
}
