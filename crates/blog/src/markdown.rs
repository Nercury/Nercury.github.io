//! Markdown → HTML.
//!
//! pulldown-cmark does the parsing; this module rewrites headings (kramdown-compatible ids, so
//! existing deep links keep working, plus an anchor link) and code blocks (highlighted at build
//! time, see `highlight.rs`).

use std::collections::HashMap;
use std::ops::Range;

use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, html};

use crate::highlight::Highlighter;

/// GitHub's "link" octicon (MIT), shown next to a heading on hover.
const ANCHOR_ICON: &str = r#"<svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><path fill="currentColor" d="M7.775 3.275a.75.75 0 001.06 1.06l1.25-1.25a2 2 0 112.83 2.83l-2.5 2.5a2 2 0 01-2.83 0 .75.75 0 00-1.06 1.06 3.5 3.5 0 004.95 0l2.5-2.5a3.5 3.5 0 00-4.95-4.95l-1.25 1.25zm-4.69 9.64a2 2 0 010-2.83l2.5-2.5a2 2 0 012.83 0 .75.75 0 001.06-1.06 3.5 3.5 0 00-4.95 0l-2.5 2.5a3.5 3.5 0 004.95 4.95l1.25-1.25a.75.75 0 00-1.06-1.06l-1.25 1.25a2 2 0 01-2.83 0z"/></svg>"#;

/// Where the HTML is going to be shown.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    /// A page of the site: heading anchors and highlighted code, styled by the site's CSS.
    Page,
    /// Embedded in the feed, where no stylesheet applies: the hover-only anchor icon would be a
    /// permanent link before every heading, and token spans would only double the feed's size.
    Feed,
}

pub fn render(source: &str, highlighter: &Highlighter, flavor: Flavor) -> Result<String> {
    let options = Options::ENABLE_TABLES | Options::ENABLE_SMART_PUNCTUATION | Options::ENABLE_STRIKETHROUGH;
    let events: Vec<(Event, Range<usize>)> = Parser::new_ext(source, options).into_offset_iter().collect();
    let mut ids = HeadingIds::default();
    let events = rewrite(source, events, &mut ids, highlighter, flavor)?;

    let mut out = String::with_capacity(source.len() * 2);
    html::push_html(&mut out, events.into_iter());
    Ok(out)
}

/// Replaces headings and code blocks with pre-rendered HTML events.
fn rewrite<'a>(
    source: &str,
    events: Vec<(Event<'a>, Range<usize>)>,
    ids: &mut HeadingIds,
    highlighter: &Highlighter,
    flavor: Flavor,
) -> Result<Vec<Event<'a>>> {
    let mut out = Vec::with_capacity(events.len());
    let mut events = events.into_iter();
    while let Some((event, range)) = events.next() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let inner: Vec<_> = events.by_ref().take_while(|(e, _)| !matches!(e, Event::End(TagEnd::Heading(_)))).collect();
                let mut inner_html = String::new();
                html::push_html(&mut inner_html, rewrite(source, inner, ids, highlighter, flavor)?.into_iter());
                let id = ids.next(heading_raw_text(&source[range]));
                let anchor = match flavor {
                    Flavor::Page => format!("<a class=\"header-link\" href=\"#{id}\" aria-label=\"Link to this section\">{ANCHOR_ICON}</a>"),
                    Flavor::Feed => String::new(),
                };
                out.push(Event::Html(format!("<{level} id=\"{id}\">{anchor}{inner_html}</{level}>\n").into()));
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let mut code = String::new();
                for (event, _) in events.by_ref() {
                    match event {
                        Event::Text(text) => code.push_str(&text),
                        Event::End(TagEnd::CodeBlock) => break,
                        _ => {}
                    }
                }
                let language = match &kind {
                    CodeBlockKind::Fenced(info) => info.split_whitespace().next().unwrap_or(""),
                    CodeBlockKind::Indented => "",
                };
                let block = match flavor {
                    Flavor::Page => highlighter.code_block(language, &code)?,
                    Flavor::Feed => Highlighter::plain_code_block(language, &code),
                };
                out.push(Event::Html(block.into()));
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

/// kramdown's GFM parser derives ids from the heading's *source* text: lowercase, drop everything
/// but word characters, `-`, space and tab, then spaces → `-`. Repeats get `-1`, `-2`, ...
#[derive(Default)]
struct HeadingIds {
    seen: HashMap<String, usize>,
}

impl HeadingIds {
    fn next(&mut self, raw_text: &str) -> String {
        let mut id: String = raw_text
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | ' ' | '\t'))
            .map(|c| if c == ' ' || c == '\t' { '-' } else { c })
            .collect();
        let count = self.seen.entry(id.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            id.push_str(&format!("-{}", *count - 1));
        }
        id
    }
}

/// The heading's text as written: without the ATX `#` markers or the setext underline.
fn heading_raw_text(block: &str) -> &str {
    let first_line = block.lines().next().unwrap_or("");
    if first_line.trim_start().starts_with('#') {
        first_line.trim().trim_start_matches('#').trim_end_matches('#').trim()
    } else {
        // setext: every line except the underline
        block.trim_end().rsplit_once('\n').map_or(block, |(text, _)| text).trim()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_ids_follow_kramdown_gfm() {
        let mut ids = HeadingIds::default();
        assert_eq!(ids.next("1. Connect the wires"), "1-connect-the-wires");
        assert_eq!(ids.next("Why Rust"), "why-rust");
        assert_eq!(ids.next("Why Rust"), "why-rust-1");
        assert_eq!(ids.next("`mem::transmute` & friends"), "memtransmute--friends");
    }

    #[test]
    fn headings_get_id_and_anchor_code_is_highlighted() {
        let highlighter = Highlighter::new("GitHub").unwrap();
        let source = "## Why `Rust`?\n\nUse `x`.\n\n```rust\nlet a = 1 < 2;\n```\n\n    indented\n";
        let html = render(source, &highlighter, Flavor::Page).unwrap();
        assert!(html.contains("<h2 id=\"why-rust\"><a class=\"header-link\" href=\"#why-rust\""), "{html}");
        assert!(html.contains("</a>Why <code>Rust</code>?</h2>"), "{html}");
        assert!(html.contains("<p>Use <code>x</code>.</p>"));
        assert!(html.contains("<pre class=\"highlight s-code\"><code class=\"language-rust\"><span"));
        assert!(html.contains("<code class=\"language-plaintext\">"));

        let feed = render(source, &highlighter, Flavor::Feed).unwrap();
        assert!(feed.contains("<h2 id=\"why-rust\">Why <code>Rust</code>?</h2>"), "{feed}");
        assert!(!feed.contains("header-link") && !feed.contains("<svg") && !feed.contains("<span"));
        assert!(feed.contains("<pre><code class=\"language-rust\">let a = 1 &lt; 2;\n</code></pre>"), "{feed}");
    }
}
