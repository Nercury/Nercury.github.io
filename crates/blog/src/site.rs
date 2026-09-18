//! Source discovery: what is a post, a page or a static file, and where each one ends up.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{Datelike, NaiveDateTime};

use crate::config::Config;
use crate::front_matter::{self, FrontMatter};

pub struct Site {
    pub root: PathBuf,
    pub config: Config,
    /// Newest first, like Jekyll's `site.posts`.
    pub posts: Vec<Document>,
    pub pages: Vec<Document>,
    /// Root-relative paths, copied verbatim.
    pub statics: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Post,
    Draft,
    Page,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Markdown,
    Scss,
    /// HTML, XML, ...: only templating applies.
    Plain,
}

pub struct Document {
    pub kind: Kind,
    pub format: Format,
    /// Root-relative source path with `/` separators.
    pub source: String,
    pub front: FrontMatter,
    pub body: String,
    pub date: Option<NaiveDateTime>,
    /// Site-relative URL, starts with `/`.
    pub url: String,
}

impl Document {
    /// Output file for the URL: directory URLs get an `index.html`, extensionless permalinks
    /// (`/resources/mcu-02/cortex-m-basics`) get `.html`, which the web server resolves.
    pub fn output_path(&self) -> String {
        let path = self.url.trim_start_matches('/');
        let file_name = path.rsplit('/').next().unwrap_or("");
        if file_name.is_empty() {
            format!("{path}index.html")
        } else if !file_name.contains('.') {
            format!("{path}.html")
        } else {
            path.to_string()
        }
    }
}

impl Site {
    pub fn load(root: &Path, drafts: bool) -> Result<Self> {
        let config = Config::load(root)?;
        let mut site = Site { root: root.to_path_buf(), config, posts: Vec::new(), pages: Vec::new(), statics: Vec::new() };

        let mut files = Vec::new();
        walk(root, root, &site.config.exclude, &mut files)?;
        for file in files {
            site.classify(file)?;
        }

        site.load_collection("_posts", Kind::Post)?;
        if drafts {
            site.load_collection("_drafts", Kind::Draft)?;
        }
        // Jekyll orders posts by date, then by source path; `site.posts` is that order reversed.
        site.posts.sort_by(|a, b| (b.date, &b.source).cmp(&(a.date, &a.source)));
        site.pages.sort_by(|a, b| a.source.cmp(&b.source));
        site.statics.sort();
        Ok(site)
    }

    /// A file outside `_posts`/`_drafts`: a page when it has front matter (or is Markdown, which
    /// jekyll-optional-front-matter renders regardless), otherwise static.
    fn classify(&mut self, source: String) -> Result<()> {
        let format = format_of(&source);
        if format == Format::Plain && !has_front_matter(&self.root.join(&source))? {
            self.statics.push(source);
            return Ok(());
        }
        let text = read(&self.root.join(&source))?;
        let (front, body) = match front_matter::split(&text).with_context(|| source.clone())? {
            Some((front, body)) => (front, body.to_string()),
            None if format == Format::Markdown => (FrontMatter::default(), text),
            None => {
                self.statics.push(source);
                return Ok(());
            }
        };
        let url = match &front.permalink {
            Some(permalink) => permalink.clone(),
            None => format!("/{}", with_output_extension(&source, format)),
        };
        let date = front.date().with_context(|| source.clone())?;
        self.pages.push(Document { kind: Kind::Page, format, source, front, body, date, url });
        Ok(())
    }

    fn load_collection(&mut self, dir: &str, kind: Kind) -> Result<()> {
        let path = self.root.join(dir);
        if !path.is_dir() {
            return Ok(());
        }
        let mut names: Vec<String> = std::fs::read_dir(&path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|name| !name.starts_with('.'))
            .collect();
        names.sort();

        for name in names {
            let source = format!("{dir}/{name}");
            let text = read(&path.join(&name))?;
            let Some((front, body)) = front_matter::split(&text).with_context(|| source.clone())? else {
                bail!("{source}: posts need front matter");
            };
            let (file_date, slug) = split_post_name(&name).with_context(|| format!("{source}: expected YYYY-MM-DD-slug.ext"))?;
            let date = match front.date().with_context(|| source.clone())? {
                Some(date) => date,
                None => front_matter::parse_date(file_date)?,
            };
            let url = post_url(&front.categories(), date, slug);
            let document = Document { kind, format: format_of(&source), source, body: body.to_string(), front, date: Some(date), url };
            self.posts.push(document);
        }
        Ok(())
    }
}

/// Jekyll's default post permalink, `/:categories/:year/:month/:day/:title.html`. Categories are
/// lowercased and de-duplicated but not slugified (`C++` → `c++`); the date comes from front
/// matter when present, not from the file name.
pub fn post_url(categories: &[String], date: NaiveDateTime, slug: &str) -> String {
    let mut segments: Vec<String> = Vec::new();
    for category in categories.iter().map(|c| c.to_lowercase()) {
        if !segments.contains(&category) {
            segments.push(category);
        }
    }
    segments.push(format!("{:04}/{:02}/{:02}", date.year(), date.month(), date.day()));
    segments.push(format!("{slug}.html"));
    format!("/{}", segments.join("/"))
}

/// `2018-02-08-some-title.markdown` → (`2018-02-08`, `some-title`)
fn split_post_name(name: &str) -> Option<(&str, &str)> {
    let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
    let date = stem.get(..10)?;
    let slug = stem.get(10..)?.strip_prefix('-')?;
    let well_formed = date.bytes().enumerate().all(|(i, b)| if i == 4 || i == 7 { b == b'-' } else { b.is_ascii_digit() });
    (well_formed && !slug.is_empty()).then_some((date, slug))
}

fn format_of(source: &str) -> Format {
    match source.rsplit_once('.').map(|(_, ext)| ext.to_ascii_lowercase()).as_deref() {
        Some("md" | "markdown" | "mkd" | "mkdown") => Format::Markdown,
        Some("scss") => Format::Scss,
        _ => Format::Plain,
    }
}

fn with_output_extension(source: &str, format: Format) -> String {
    let extension = match format {
        Format::Markdown => "html",
        Format::Scss => "css",
        Format::Plain => return source.to_string(),
    };
    let stem = source.rsplit_once('.').map_or(source, |(stem, _)| stem);
    format!("{stem}.{extension}")
}

fn has_front_matter(path: &Path) -> Result<bool> {
    use std::io::Read;
    let mut start = [0u8; 4];
    let mut file = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let read = file.read(&mut start)?;
    Ok(start[..read].starts_with(b"---\n") || start[..read].starts_with(b"---\r"))
}

fn read(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))
}

/// Collects root-relative file paths, skipping what Jekyll skips plus the configured excludes.
fn walk(root: &Path, dir: &Path, exclude: &[String], out: &mut Vec<String>) -> Result<()> {
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.starts_with(['_', '.', '#']) || name.ends_with('~') {
            continue;
        }
        let relative = path.strip_prefix(root)?.to_string_lossy().replace('\\', "/");
        if exclude.iter().any(|e| e.trim_matches('/') == relative) {
            continue;
        }
        if path.is_dir() {
            walk(root, &path, exclude, out)?;
        } else {
            out.push(relative);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(text: &str) -> NaiveDateTime {
        front_matter::parse_date(text).unwrap()
    }

    #[test]
    fn post_urls_match_jekyll() {
        let categories = |text: &str| text.split_whitespace().map(str::to_string).collect::<Vec<_>>();
        assert_eq!(
            post_url(&categories("C++ interesting"), date("2016-02-22"), "weak_ptr-and-event-cleanup"),
            "/c++/interesting/2016/02/22/weak_ptr-and-event-cleanup.html"
        );
        assert_eq!(
            post_url(&categories("rust pointers"), date("2014-10-30 19:38:03"), "rust-mem-transmute-and-void-pointers"),
            "/rust/pointers/2014/10/30/rust-mem-transmute-and-void-pointers.html"
        );
        assert_eq!(post_url(&[], date("2020-01-05"), "x"), "/2020/01/05/x.html");
    }

    #[test]
    fn post_file_names() {
        assert_eq!(split_post_name("2019-11-09-rust-embedded-hack-the-crab.markdown"), Some(("2019-11-09", "rust-embedded-hack-the-crab")));
        assert_eq!(split_post_name("notes.md"), None);
        assert_eq!(split_post_name("2019-11-09-.md"), None);
    }
}
