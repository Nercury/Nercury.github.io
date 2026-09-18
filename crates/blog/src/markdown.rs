//! Markdown → HTML, shaped like the kramdown (GFM input) + Rouge output of the old Jekyll build.
//!
//! pulldown-cmark does the parsing; this module rewrites the parts where kramdown's HTML differs:
//! heading ids, code block wrappers, inline code classes and the `{% highlight %}` Liquid tag.
//! Code is emitted as plain escaped text: highlighting happens client-side (MIGRATION D4).

use std::collections::HashMap;
use std::ops::Range;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, html};

/// Languages Rouge had a lexer for, as observed in the reference site. Fences with any other
/// info string (`txt`, `plain`, `gdb`, `cfg`, `C`) were left as a bare `<pre><code>`.
const ROUGE_LANGUAGES: &[&str] =
    &["bash", "c", "c++", "console", "cpp", "cs", "csharp", "glsl", "haskell", "python", "rust", "text", "toml", "plaintext"];

pub fn render(source: &str) -> String {
    let (source, highlights) = extract_highlight_tags(source);

    let options = Options::ENABLE_TABLES | Options::ENABLE_SMART_PUNCTUATION | Options::ENABLE_STRIKETHROUGH;
    let events: Vec<(Event, Range<usize>)> = Parser::new_ext(&source, options).into_offset_iter().collect();
    let mut ids = HeadingIds::default();
    let events = rewrite(&source, events, &mut ids);

    let mut out = String::with_capacity(source.len() * 2);
    html::push_html(&mut out, events.into_iter());

    for (index, block) in highlights.iter().enumerate() {
        out = out.replace(&highlight_placeholder(index), block);
    }
    out
}

/// Replaces headings, code blocks and inline code with pre-rendered HTML events.
fn rewrite<'a>(source: &str, events: Vec<(Event<'a>, Range<usize>)>, ids: &mut HeadingIds) -> Vec<Event<'a>> {
    let mut out = Vec::with_capacity(events.len());
    let mut events = events.into_iter();
    while let Some((event, range)) = events.next() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let inner: Vec<_> = events.by_ref().take_while(|(e, _)| !matches!(e, Event::End(TagEnd::Heading(_)))).collect();
                let mut inner_html = String::new();
                html::push_html(&mut inner_html, rewrite(source, inner, ids).into_iter());
                let id = ids.next(heading_raw_text(&source[range]));
                out.push(Event::Html(format!("<{level} id=\"{id}\">{inner_html}</{level}>\n").into()));
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
                out.push(Event::Html(code_block(language, &code).into()));
            }
            Event::Code(code) => {
                out.push(Event::Html(format!("<code class=\"language-plaintext highlighter-rouge\">{}</code>", escape(&code)).into()));
            }
            other => out.push(other),
        }
    }
    out
}

fn code_block(language: &str, code: &str) -> String {
    let code = escape(code);
    if language.is_empty() || ROUGE_LANGUAGES.contains(&language) {
        let language = if language.is_empty() { "plaintext" } else { language };
        format!("<div class=\"language-{language} highlighter-rouge\"><div class=\"highlight\"><pre class=\"highlight\"><code>{code}</code></pre></div></div>\n")
    } else {
        format!("<pre><code class=\"language-{language}\">{code}</code></pre>\n")
    }
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

/// Jekyll's `{% highlight lang %}` runs before Markdown and leaves a `<figure>` block behind.
/// The tag bodies are lifted out here and spliced back into the final HTML, so Markdown never
/// sees the code.
fn extract_highlight_tags(source: &str) -> (String, Vec<String>) {
    let mut out = String::with_capacity(source.len());
    let mut blocks = Vec::new();
    let mut rest = source;
    while let Some(start) = rest.find("{% highlight ") {
        let after_open = &rest[start + "{% highlight ".len()..];
        let (Some(open_end), Some(close)) = (after_open.find("%}"), after_open.find("{% endhighlight %}")) else { break };
        if open_end > close {
            break;
        }
        let language = after_open[..open_end].split_whitespace().next().unwrap_or("text");
        // Jekyll strips leading and trailing newlines of the tag body.
        let code = after_open[open_end + 2..close].trim_matches(['\n', '\r']);
        out.push_str(&rest[..start]);
        out.push_str(&highlight_placeholder(blocks.len()));
        blocks.push(format!(
            "<figure class=\"highlight\"><pre><code class=\"language-{language}\" data-lang=\"{language}\">{}</code></pre></figure>",
            escape(code)
        ));
        rest = &after_open[close + "{% endhighlight %}".len()..];
    }
    out.push_str(rest);
    (out, blocks)
}

/// An HTML comment survives Markdown untouched, both as a block and inline.
fn highlight_placeholder(index: usize) -> String {
    format!("<!--highlight-block-{index}-->")
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
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
    fn code_blocks_by_language() {
        let html = render("```rust\nlet a = 1 < 2;\n```\n\n```txt\nplain\n```\n\n```\nnone\n```\n");
        assert!(html.contains("<div class=\"language-rust highlighter-rouge\"><div class=\"highlight\"><pre class=\"highlight\"><code>let a = 1 &lt; 2;\n</code>"));
        assert!(html.contains("<pre><code class=\"language-txt\">plain\n</code></pre>"));
        assert!(html.contains("<div class=\"language-plaintext highlighter-rouge\">"));
    }

    #[test]
    fn highlight_tag_becomes_figure() {
        let html = render("Before\n\n{% highlight rust %}\nfn a() {}\n\nfn b<T>() {}\n{% endhighlight %}\n\nAfter `x`\n");
        assert!(html.contains("<figure class=\"highlight\"><pre><code class=\"language-rust\" data-lang=\"rust\">fn a() {}\n\nfn b&lt;T&gt;() {}</code></pre></figure>"));
        assert!(html.contains("<code class=\"language-plaintext highlighter-rouge\">x</code>"));
    }
}
