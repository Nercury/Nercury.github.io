//! Writes the site into the output directory.

use std::path::Path;

use anyhow::{Context, Result, bail};

use liquid::Object;
use liquid::model::Value;

use crate::markdown;
use crate::site::{Format, Site};
use crate::templates::{self, Templates};

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

    let templates = Templates::load(&site.root)?;

    // Posts first: `site.posts[].content` must exist before any template runs. Liquid is not
    // applied to Markdown sources (the only tag they use, `highlight`, is handled by the
    // Markdown renderer).
    let post_content: Vec<String> = site.posts.iter().map(|post| markdown::render(&post.body)).collect();

    let mut site_object = templates::site_object(&site.config);
    let posts = site.posts.iter().zip(&post_content).map(|(post, content)| Value::Object(templates::document_object(post, Some(content))));
    site_object.insert("posts".into(), Value::Array(posts.collect()));
    let pages = site.pages.iter().map(|page| Value::Object(templates::document_object(page, None)));
    site_object.insert("pages".into(), Value::Array(pages.collect()));
    let site_value = Value::Object(site_object);

    let mut seen = std::collections::BTreeMap::new();
    let documents = site.posts.iter().zip(post_content.iter().map(Some)).chain(site.pages.iter().map(|page| (page, None)));
    let mut count = 0;
    for (document, content) in documents {
        let path = document.output_path();
        if let Some(other) = seen.insert(path.clone(), &document.source) {
            bail!("{} and {} both write {path}", other, document.source);
        }

        let mut globals = Object::new();
        globals.insert("site".into(), site_value.clone());
        globals.insert("page".into(), Value::Object(templates::document_object(document, content.map(String::as_str))));

        let output = (|| {
            let content = match (content, document.format) {
                (Some(content), _) => content.clone(),
                (None, Format::Markdown) => markdown::render(&document.body),
                (None, Format::Scss) => return scss(site, &document.body),
                (None, Format::Plain) => templates.render_body(&document.body, &globals)?,
            };
            templates.apply_layouts(document.front.layout.as_deref(), content, &globals)
        })()
        .with_context(|| format!("rendering {}", document.source))?;

        let target = out.join(&path);
        create_parent(&target)?;
        std::fs::write(&target, output).with_context(|| format!("writing {}", target.display()))?;
        count += 1;
    }

    Ok(Summary { documents: count, statics: site.statics.len() })
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
