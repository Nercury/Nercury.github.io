//! Explicit list of tolerated differences (`sitediff.allow.toml`).
//!
//! Three rule kinds, each with a mandatory `reason`:
//! - `[[file]]`  — ignore a path entirely (missing, extra or different).
//! - `[[drop]]`  — remove elements matching a CSS selector before HTML comparison.
//! - `[[line]]`  — tolerate changed lines of the normalized form that match a regex.

use std::cell::Cell;
use std::path::Path;

use anyhow::{Context, Result, bail};
use globset::{Glob, GlobMatcher};
use regex::Regex;
use scraper::Selector;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Reference,
    Candidate,
    #[default]
    Both,
}

impl Side {
    pub fn covers(self, other: Side) -> bool {
        self == Side::Both || self == other
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    #[serde(default)]
    file: Vec<RawRule>,
    #[serde(default)]
    drop: Vec<RawRule>,
    #[serde(default)]
    line: Vec<RawRule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    path: String,
    reason: String,
    #[serde(default)]
    side: Side,
    selector: Option<String>,
    contains: Option<String>,
    pattern: Option<String>,
}

pub struct FileRule {
    pub common: Common,
}

pub struct DropRule {
    pub common: Common,
    pub selector: Selector,
    pub contains: Option<String>,
}

pub struct LineRule {
    pub common: Common,
    pub pattern: Regex,
}

pub struct Common {
    pub label: String,
    pub reason: String,
    pub side: Side,
    path: GlobMatcher,
    hits: Cell<usize>,
}

impl Common {
    pub fn matches_path(&self, path: &str) -> bool {
        self.path.is_match(path)
    }

    pub fn hit(&self) {
        self.hits.set(self.hits.get() + 1);
    }
}

#[derive(Default)]
pub struct AllowList {
    pub files: Vec<FileRule>,
    pub drops: Vec<DropRule>,
    pub lines: Vec<LineRule>,
}

impl AllowList {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        Self::parse(&text).with_context(|| format!("parsing {}", path.display()))
    }

    pub fn parse(text: &str) -> Result<Self> {
        let raw: RawFile = toml::from_str(text)?;
        let mut list = AllowList::default();
        for (i, r) in raw.file.into_iter().enumerate() {
            list.files.push(FileRule { common: common("file", i, &r)? });
        }
        for (i, r) in raw.drop.into_iter().enumerate() {
            let common = common("drop", i, &r)?;
            let Some(selector) = &r.selector else { bail!("{}: `selector` is required", common.label) };
            let selector = Selector::parse(selector)
                .map_err(|e| anyhow::anyhow!("{}: bad selector: {e}", common.label))?;
            list.drops.push(DropRule { common, selector, contains: r.contains });
        }
        for (i, r) in raw.line.into_iter().enumerate() {
            let common = common("line", i, &r)?;
            let Some(pattern) = &r.pattern else { bail!("{}: `pattern` is required", common.label) };
            let pattern = Regex::new(pattern).with_context(|| format!("{}: bad pattern", common.label))?;
            list.lines.push(LineRule { common, pattern });
        }
        Ok(list)
    }

    pub fn file_ignored(&self, path: &str) -> bool {
        let mut ignored = false;
        for rule in self.files.iter().filter(|r| r.common.matches_path(path)) {
            rule.common.hit();
            ignored = true;
        }
        ignored
    }

    /// Is a changed line on `side` of `path` tolerated?
    pub fn line_allowed(&self, path: &str, side: Side, line: &str) -> bool {
        let mut allowed = false;
        for rule in &self.lines {
            if rule.common.side.covers(side) && rule.common.matches_path(path) && rule.pattern.is_match(line) {
                rule.common.hit();
                allowed = true;
            }
        }
        allowed
    }

    /// Labels and reasons of rules that never matched anything.
    pub fn unused(&self) -> Vec<String> {
        let all = (self.files.iter().map(|r| &r.common))
            .chain(self.drops.iter().map(|r| &r.common))
            .chain(self.lines.iter().map(|r| &r.common));
        all.filter(|c| c.hits.get() == 0).map(|c| format!("{} ({})", c.label, c.reason)).collect()
    }
}

fn common(kind: &str, index: usize, raw: &RawRule) -> Result<Common> {
    let label = format!("[[{kind}]] #{} path={:?}", index + 1, raw.path);
    if raw.reason.trim().is_empty() {
        bail!("{label}: `reason` must not be empty");
    }
    let path = Glob::new(&raw.path).with_context(|| format!("{label}: bad glob"))?.compile_matcher();
    Ok(Common { label, reason: raw.reason.clone(), side: raw.side, path, hits: Cell::new(0) })
}
