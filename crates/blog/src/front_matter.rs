//! YAML front matter: the `---` delimited block at the top of a source file.

use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct FrontMatter {
    pub layout: Option<String>,
    pub title: Option<String>,
    date: Option<String>,
    #[serde(default)]
    categories: Words,
    #[serde(default)]
    tags: Words,
    pub permalink: Option<String>,
    /// Any other key; exposed to templates as `page.<key>`.
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, serde_yaml_ng::Value>,
}

/// Jekyll accepts both `categories: a b` and `categories: [a, b]`.
#[derive(Debug, Default, Deserialize)]
#[serde(untagged)]
enum Words {
    #[default]
    None,
    Text(String),
    List(Vec<String>),
}

impl Words {
    fn to_vec(&self) -> Vec<String> {
        match self {
            Words::None => Vec::new(),
            Words::Text(text) => text.split_whitespace().map(str::to_string).collect(),
            Words::List(list) => list.clone(),
        }
    }
}

impl FrontMatter {
    pub fn categories(&self) -> Vec<String> {
        self.categories.to_vec()
    }

    pub fn tags(&self) -> Vec<String> {
        self.tags.to_vec()
    }

    /// `2014-10-30`, `2014-10-30 19:38:03` or `2018-09-30 11:00:00 UTC`. Times are treated as UTC.
    pub fn date(&self) -> Result<Option<NaiveDateTime>> {
        self.date.as_deref().map(parse_date).transpose()
    }
}

pub fn parse_date(text: &str) -> Result<NaiveDateTime> {
    let mut parts = text.split_whitespace();
    let date = parts.next().unwrap_or("");
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").with_context(|| format!("bad date {text:?}"))?;
    let time = match parts.next() {
        Some(time) => NaiveTime::parse_from_str(time, "%H:%M:%S").with_context(|| format!("bad time in {text:?}"))?,
        None => NaiveTime::MIN,
    };
    Ok(date.and_time(time))
}

/// Splits `---\n<yaml>\n---\n<body>`. `None` when the file has no front matter.
pub fn split(source: &str) -> Result<Option<(FrontMatter, &str)>> {
    let Some(rest) = source.strip_prefix("---") else { return Ok(None) };
    let Some(rest) = rest.strip_prefix("\r\n").or_else(|| rest.strip_prefix('\n')) else { return Ok(None) };

    // The closing delimiter is a line consisting of `---`; an empty block closes immediately.
    let (yaml, body) = if let Some(body) = rest.strip_prefix("---") {
        ("", body)
    } else {
        let Some(end) = rest.find("\n---") else { return Ok(None) };
        (&rest[..end], &rest[end + 4..])
    };
    let body = body.strip_prefix("\r\n").or_else(|| body.strip_prefix('\n')).unwrap_or(body);

    let front = if yaml.trim().is_empty() || yaml.lines().all(|l| l.trim().is_empty() || l.trim_start().starts_with('#')) {
        FrontMatter::default()
    } else {
        serde_yaml_ng::from_str(yaml).context("parsing front matter")?
    };
    Ok(Some((front, body)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_front_matter_and_body() {
        let (front, body) = split("---\nlayout: post\ntitle:  \"A: b\"\ncategories: rust C++\n---\n\nBody\n").unwrap().unwrap();
        assert_eq!(front.layout.as_deref(), Some("post"));
        assert_eq!(front.title.as_deref(), Some("A: b"));
        assert_eq!(front.categories(), ["rust", "C++"]);
        assert_eq!(body, "\nBody\n");
    }

    #[test]
    fn empty_and_comment_only_front_matter() {
        let (front, body) = split("---\n---\nbody").unwrap().unwrap();
        assert!(front.layout.is_none());
        assert_eq!(body, "body");
        let (_, body) = split("---\n# just a comment\n---\n@charset \"utf-8\";").unwrap().unwrap();
        assert_eq!(body, "@charset \"utf-8\";");
    }

    #[test]
    fn no_front_matter() {
        assert!(split("# Heading\n\ntext --- text").unwrap().is_none());
    }

    #[test]
    fn date_forms() {
        assert_eq!(parse_date("2014-11-02").unwrap().to_string(), "2014-11-02 00:00:00");
        assert_eq!(parse_date("2014-10-30 19:38:03").unwrap().to_string(), "2014-10-30 19:38:03");
        assert_eq!(parse_date("2018-09-30 11:00:00 UTC").unwrap().to_string(), "2018-09-30 11:00:00");
    }
}
