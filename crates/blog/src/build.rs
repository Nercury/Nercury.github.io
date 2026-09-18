//! Writes the site into the output directory.

use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::site::{Document, Format, Site};

pub struct Summary {
    pub documents: usize,
    pub statics: usize,
}

pub fn build(site: &Site, out: &Path) -> Result<Summary> {
    prepare_output_dir(out)?;

    for source in &site.statics {
        let target = out.join(source);
        create_parent(&target)?;
        std::fs::copy(site.root.join(source), &target).with_context(|| format!("copying {source}"))?;
    }

    let documents: Vec<&Document> = site.posts.iter().chain(&site.pages).collect();
    let mut seen = std::collections::BTreeMap::new();
    for document in &documents {
        let path = document.output_path();
        if let Some(other) = seen.insert(path.clone(), &document.source) {
            bail!("{} and {} both write {path}", other, document.source);
        }
        let content = render(site, document).with_context(|| format!("rendering {}", document.source))?;
        let target = out.join(&path);
        create_parent(&target)?;
        std::fs::write(&target, content).with_context(|| format!("writing {}", target.display()))?;
    }

    Ok(Summary { documents: documents.len(), statics: site.statics.len() })
}

fn render(site: &Site, document: &Document) -> Result<String> {
    match document.format {
        Format::Scss => scss(site, &document.body),
        // Placeholders until templates (Phase 4) and Markdown (Phase 5) land.
        _ if document.output_path().ends_with(".xml") => Ok("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\"><channel></channel></rss>\n".to_string()),
        _ => Ok(format!("<!DOCTYPE html>\n<title>{}</title>\n", document.front.title.as_deref().unwrap_or(&site.config.title))),
    }
}

/// Jekyll on GitHub Pages compiles with `style: compressed` and `_sass` as the load path.
fn scss(site: &Site, source: &str) -> Result<String> {
    let options = grass::Options::default().style(grass::OutputStyle::Compressed).load_path(site.root.join("_sass"));
    grass::from_string(source, &options).map_err(|e| anyhow::anyhow!("{e}"))
}

/// Start from an empty directory, but never delete something that does not look like build output.
fn prepare_output_dir(out: &Path) -> Result<()> {
    if out.exists() {
        for marker in [".git", "site.toml", "Cargo.toml"] {
            if out.join(marker).exists() {
                bail!("refusing to clear {}: it contains {marker}", out.display());
            }
        }
        std::fs::remove_dir_all(out).with_context(|| format!("clearing {}", out.display()))?;
    }
    std::fs::create_dir_all(out).with_context(|| format!("creating {}", out.display()))
}

fn create_parent(path: &Path) -> Result<()> {
    match path.parent() {
        Some(parent) => std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display())),
        None => Ok(()),
    }
}
