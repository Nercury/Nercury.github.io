//! Walks a reference and a candidate site tree and compares them file by file.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::GlobMatcher;
use sha2::{Digest, Sha256};
use similar::{ChangeTag, TextDiff};

use crate::allow::{AllowList, Side};
use crate::{css, feed, html};

pub struct Options {
    pub reference: PathBuf,
    pub candidate: PathBuf,
    /// `sha256 *path` lines for reference files that are not stored in the reference tree.
    pub manifest: Option<PathBuf>,
    /// Restrict the comparison to matching paths.
    pub only: Option<GlobMatcher>,
    pub allow: AllowList,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Same,
    /// Differs, but every changed line is covered by the allowlist.
    Allowed { lines: usize },
    /// Ignored by a `[[file]]` rule.
    Ignored,
    Different { hunks: String, lines: usize, allowed_lines: usize },
    HashMismatch,
    Missing,
    Extra,
}

impl Status {
    pub fn is_failure(&self) -> bool {
        matches!(self, Status::Different { .. } | Status::HashMismatch | Status::Missing | Status::Extra)
    }
}

pub struct Report {
    pub files: BTreeMap<String, Status>,
    pub unused_rules: Vec<String>,
}

impl Report {
    pub fn failures(&self) -> usize {
        self.files.values().filter(|s| s.is_failure()).count()
    }
}

pub fn run(options: &Options) -> Result<Report> {
    let reference = list_files(&options.reference)?;
    let candidate = list_files(&options.candidate)?;
    let manifest = match &options.manifest {
        Some(path) => read_manifest(path)?,
        None => BTreeMap::new(),
    };

    let mut paths: BTreeSet<&String> = reference.iter().chain(&candidate).chain(manifest.keys()).collect();
    if let Some(only) = &options.only {
        paths.retain(|p| only.is_match(p.as_str()));
    }

    let mut files = BTreeMap::new();
    for path in paths {
        let in_reference = reference.contains(path) || manifest.contains_key(path);
        let status = if options.allow.file_ignored(path) {
            Status::Ignored
        } else if !in_reference {
            Status::Extra
        } else if !candidate.contains(path) {
            Status::Missing
        } else if reference.contains(path) {
            compare_file(path, options).with_context(|| format!("comparing {path}"))?
        } else if sha256(&options.candidate.join(path))? == manifest[path] {
            Status::Same
        } else {
            Status::HashMismatch
        };
        files.insert(path.clone(), status);
    }
    Ok(Report { files, unused_rules: options.allow.unused() })
}

fn compare_file(path: &str, options: &Options) -> Result<Status> {
    let reference = std::fs::read(options.reference.join(path))?;
    let candidate = std::fs::read(options.candidate.join(path))?;
    let extension = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("");

    // Drop rules can make identical HTML files relevant (unused-rule tracking), so HTML/XML always
    // go through normalization; everything else short-circuits on identical bytes.
    let (old, new) = match extension {
        "html" | "htm" => {
            let side = |bytes: &[u8], side| html::document_lines(&String::from_utf8_lossy(bytes), path, side, &options.allow);
            (side(&reference, Side::Reference), side(&candidate, Side::Candidate))
        }
        "xml" => {
            let side = |bytes: &[u8], side| feed::lines(&String::from_utf8_lossy(bytes), path, side, &options.allow);
            (side(&reference, Side::Reference)?, side(&candidate, Side::Candidate)?)
        }
        _ if reference == candidate => return Ok(Status::Same),
        "css" => (css::lines(&String::from_utf8_lossy(&reference)), css::lines(&String::from_utf8_lossy(&candidate))),
        _ => return Ok(Status::HashMismatch),
    };
    Ok(diff_lines(path, &old, &new, &options.allow))
}

fn diff_lines(path: &str, old: &[String], new: &[String], allow: &AllowList) -> Status {
    let old: Vec<&str> = old.iter().map(String::as_str).collect();
    let new: Vec<&str> = new.iter().map(String::as_str).collect();
    let diff = TextDiff::from_slices(&old, &new);

    let (mut hunks, mut lines, mut allowed_lines) = (String::new(), 0, 0);
    for group in diff.grouped_ops(3) {
        let mut rendered = String::new();
        let mut hunk_fails = false;
        for change in group.iter().flat_map(|op| diff.iter_changes(op)) {
            let (sign, side) = match change.tag() {
                ChangeTag::Equal => (' ', None),
                ChangeTag::Delete => ('-', Some(Side::Reference)),
                ChangeTag::Insert => ('+', Some(Side::Candidate)),
            };
            let text = change.value();
            let mut mark = ' ';
            if let Some(side) = side {
                lines += 1;
                if allow.line_allowed(path, side, text) {
                    allowed_lines += 1;
                    mark = '~';
                } else {
                    hunk_fails = true;
                }
            }
            let _ = writeln!(rendered, "  {sign}{mark}{text}");
        }
        if hunk_fails {
            let _ = writeln!(hunks, "  @@");
            hunks.push_str(&rendered);
        }
    }

    match (lines, lines == allowed_lines) {
        (0, _) => Status::Same,
        (_, true) => Status::Allowed { lines },
        _ => Status::Different { hunks, lines, allowed_lines },
    }
}

fn list_files(root: &Path) -> Result<BTreeSet<String>> {
    fn visit(root: &Path, dir: &Path, out: &mut BTreeSet<String>) -> Result<()> {
        for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
            let path = entry?.path();
            if path.is_dir() {
                visit(root, &path, out)?;
            } else {
                let relative = path.strip_prefix(root)?.to_string_lossy().replace('\\', "/");
                out.insert(relative);
            }
        }
        Ok(())
    }
    let mut out = BTreeSet::new();
    visit(root, root, &mut out)?;
    Ok(out)
}

/// Parses `sha256sum` output: `<hex>  <path>` or `<hex> *<path>`.
fn read_manifest(path: &Path) -> Result<BTreeMap<String, String>> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut out = BTreeMap::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let (hash, file) = line.split_once(' ').with_context(|| format!("bad manifest line: {line}"))?;
        let file = file.strip_prefix(['*', ' ']).unwrap_or(file);
        out.insert(file.to_string(), hash.to_ascii_lowercase());
    }
    Ok(out)
}

fn sha256(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect())
}
