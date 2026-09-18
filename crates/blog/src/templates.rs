//! Liquid rendering: `_includes` as partials, `_layouts` as a chain of wrappers around `content`.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use chrono::NaiveDateTime;
use liquid::model::{DateTime, Value};
use liquid::partials::{EagerCompiler, InMemorySource};
use liquid::{Object, Parser, Template};

use crate::config::Config;
use crate::filters;
use crate::front_matter;
use crate::site::Document;

pub struct Templates {
    parser: Parser,
    layouts: BTreeMap<String, Layout>,
}

struct Layout {
    parent: Option<String>,
    template: Template,
}

impl Templates {
    pub fn load(root: &Path) -> Result<Self> {
        let mut partials = EagerCompiler::<InMemorySource>::empty();
        for (name, text) in read_dir(&root.join("_includes"))? {
            partials.add(name, text);
        }
        let parser = liquid::ParserBuilder::with_stdlib()
            .tag(liquid_lib::jekyll::IncludeTag)
            .filter(filters::XmlEscape)
            .filter(filters::DateToRfc822)
            .partials(partials)
            .build()
            .map_err(|e| anyhow!("{e}"))?;

        let mut layouts = BTreeMap::new();
        for (file, text) in read_dir(&root.join("_layouts"))? {
            let (front, body) = match front_matter::split(&text).with_context(|| format!("_layouts/{file}"))? {
                Some((front, body)) => (front, body.to_string()),
                None => (Default::default(), text),
            };
            let template = parser.parse(&body).map_err(|e| anyhow!("_layouts/{file}: {e}"))?;
            let name = file.rsplit_once('.').map_or(file.as_str(), |(stem, _)| stem).to_string();
            layouts.insert(name, Layout { parent: front.layout, template });
        }
        Ok(Templates { parser, layouts })
    }

    /// Renders Liquid in a document body (`index.html`, `feed.xml`).
    pub fn render_body(&self, source: &str, globals: &Object) -> Result<String> {
        let template = self.parser.parse(source).map_err(|e| anyhow!("{e}"))?;
        template.render(globals).map_err(|e| anyhow!("{e}"))
    }

    /// Wraps `content` in the layout chain starting at `layout`.
    pub fn apply_layouts(&self, layout: Option<&str>, mut content: String, globals: &Object) -> Result<String> {
        let mut next = layout;
        let mut depth = 0;
        while let Some(name) = next {
            let Some(layout) = self.layouts.get(name) else { bail!("unknown layout {name:?}") };
            let mut globals = globals.clone();
            globals.insert("content".into(), Value::scalar(content));
            content = layout.template.render(&globals).map_err(|e| anyhow!("layout {name}: {e}"))?;
            next = layout.parent.as_deref();
            depth += 1;
            if depth > 16 {
                bail!("layout cycle at {name:?}");
            }
        }
        Ok(content)
    }
}

/// `site.*` without the collections, which the build adds once documents are rendered.
pub fn site_object(config: &Config) -> Object {
    let text = |value: &Option<String>| value.as_ref().map_or(Value::Nil, |v| Value::scalar(v.clone()));
    liquid::object!({
        "title": config.title,
        "email": config.email,
        "description": config.description,
        "baseurl": config.baseurl,
        "url": config.url,
        "twitter_username": text(&config.twitter_username),
        "github_username": text(&config.github_username),
        "time": Value::scalar(DateTime::now()),
    })
}

/// `page` / `post`: front matter keys plus what the generator computed.
pub fn document_object(document: &Document, content: Option<&str>) -> Object {
    let mut object = Object::new();
    // Ruby Liquid renders a missing key as nothing; this implementation errors on `{{ page.title }}`
    // when the key is absent, so the standard keys always exist.
    for key in ["title", "date", "layout", "permalink"] {
        object.insert(key.into(), Value::Nil);
    }
    for (key, value) in &document.front.extra {
        object.insert(key.clone().into(), yaml_value(value));
    }
    let front = &document.front;
    if let Some(layout) = &front.layout {
        object.insert("layout".into(), Value::scalar(layout.clone()));
    }
    if let Some(title) = &front.title {
        object.insert("title".into(), Value::scalar(title.clone()));
    }
    if let Some(permalink) = &front.permalink {
        object.insert("permalink".into(), Value::scalar(permalink.clone()));
    }
    if let Some(date) = document.date {
        object.insert("date".into(), date_value(date));
    }
    let words = |list: Vec<String>| Value::Array(list.into_iter().map(Value::scalar).collect());
    object.insert("categories".into(), words(front.categories()));
    object.insert("tags".into(), words(front.tags()));
    object.insert("url".into(), Value::scalar(document.url.clone()));
    object.insert("path".into(), Value::scalar(document.source.clone()));
    if let Some(content) = content {
        object.insert("content".into(), Value::scalar(content.to_string()));
    }
    object
}

/// Front matter times carry no zone; like the old build on GitHub, treat them as UTC.
fn date_value(date: NaiveDateTime) -> Value {
    let text = date.format("%Y-%m-%d %H:%M:%S +0000").to_string();
    DateTime::from_str(&text).map_or(Value::Nil, Value::scalar)
}

fn yaml_value(value: &serde_yaml_ng::Value) -> Value {
    use serde_yaml_ng::Value as Yaml;
    match value {
        Yaml::Null => Value::Nil,
        Yaml::Bool(b) => Value::scalar(*b),
        Yaml::Number(n) => match n.as_i64() {
            Some(i) => Value::scalar(i),
            None => Value::scalar(n.as_f64().unwrap_or(0.0)),
        },
        Yaml::String(s) => Value::scalar(s.clone()),
        Yaml::Sequence(items) => Value::Array(items.iter().map(yaml_value).collect()),
        Yaml::Mapping(map) => {
            let mut object = Object::new();
            for (key, value) in map {
                if let Some(key) = key.as_str() {
                    object.insert(key.to_string().into(), yaml_value(value));
                }
            }
            Value::Object(object)
        }
        Yaml::Tagged(tagged) => yaml_value(&tagged.value),
    }
}

/// File name → contents, for a flat directory.
fn read_dir(dir: &Path) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_file() {
            let text = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            out.push((path.file_name().unwrap_or_default().to_string_lossy().into_owned(), text));
        }
    }
    out.sort();
    Ok(out)
}
