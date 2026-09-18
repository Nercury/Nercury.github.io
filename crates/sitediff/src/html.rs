//! HTML → canonical line form.
//!
//! One line per node, indented by depth, so a plain line diff becomes a readable DOM diff.
//!
//! What is normalized away (and therefore NOT detected):
//! - whitespace runs and whitespace-only text between block elements,
//! - attribute order and class token order,
//! - entity spelling (the parser decodes them),
//! - `<span>` wrappers inside `<pre>` (highlighter tokens): code is compared by text only,
//! - the wrapper shape of a code block: Rouge's `div.highlighter-rouge > div.highlight > pre > code`,
//!   Jekyll's `figure.highlight > pre > code` and a plain `pre > code` all become one
//!   `<codeblock language="…">` line followed by the code text (trailing newlines trimmed).
//!
//! What is kept: element structure, all attributes, text, whitespace between inline elements,
//! exact text (including newlines) inside `<pre>`, comments, doctype.

use ego_tree::NodeRef;
use scraper::{Html, Node};

use crate::allow::{AllowList, Side};

const INLINE: &[&str] = &[
    "a", "abbr", "b", "bdi", "bdo", "cite", "code", "del", "dfn", "em", "i", "img", "ins", "kbd", "mark", "q", "s",
    "samp", "small", "span", "strong", "sub", "sup", "time", "u", "var",
];

pub fn document_lines(source: &str, path: &str, side: Side, allow: &AllowList) -> Vec<String> {
    lines_of(Html::parse_document(source), path, side, allow)
}

pub fn fragment_lines(source: &str, path: &str, side: Side, allow: &AllowList) -> Vec<String> {
    lines_of(Html::parse_fragment(source), path, side, allow)
}

fn lines_of(mut html: Html, path: &str, side: Side, allow: &AllowList) -> Vec<String> {
    apply_drops(&mut html, path, side, allow);
    let mut out = Vec::new();
    walk(html.tree.root(), 0, &mut out);
    out
}

fn apply_drops(html: &mut Html, path: &str, side: Side, allow: &AllowList) {
    for rule in &allow.drops {
        if !rule.common.side.covers(side) || !rule.common.matches_path(path) {
            continue;
        }
        let ids: Vec<_> = html
            .select(&rule.selector)
            .filter(|el| match &rule.contains {
                Some(needle) => el.text().collect::<String>().contains(needle.as_str()),
                None => true,
            })
            .map(|el| el.id())
            .collect();
        for id in ids {
            rule.common.hit();
            if let Some(mut node) = html.tree.get_mut(id) {
                node.detach();
            }
        }
    }
}

fn walk(node: NodeRef<Node>, depth: usize, out: &mut Vec<String>) {
    let pad = "  ".repeat(depth);
    match node.value() {
        Node::Document | Node::Fragment => node.children().for_each(|c| walk(c, depth, out)),
        Node::Doctype(d) => out.push(format!("{pad}<!DOCTYPE {}>", d.name())),
        Node::Comment(c) => out.push(format!("{pad}<!--{}-->", collapse(c))),
        Node::ProcessingInstruction(_) => {}
        Node::Text(t) => {
            let collapsed = collapse(t);
            let text = edge_trimmed(node, &collapsed);
            if !text.is_empty() {
                out.push(format!("{pad}{text:?}"));
            }
        }
        Node::Element(el) => {
            // `parse_fragment` wraps everything in a synthetic <html>.
            if el.name() == "html" && matches!(node.parent().map(|p| p.value()), Some(Node::Fragment)) {
                return node.children().for_each(|c| walk(c, depth, out));
            }
            if let Some((language, pre)) = code_block(node) {
                out.push(format!("{pad}<codeblock language={language:?}>"));
                let text: String = pre.descendants().filter_map(|n| n.value().as_text().map(|t| t.to_string())).collect();
                let pad = "  ".repeat(depth + 1);
                out.extend(text.trim_end_matches(['\n', '\r']).split('\n').map(|l| format!("{pad}|{l}")));
                return;
            }
            out.push(format!("{pad}{}", open_tag(el)));
            match el.name() {
                "pre" => {
                    let mut buffer = String::new();
                    node.children().for_each(|c| walk_pre(c, depth + 1, &mut buffer, out));
                    flush_verbatim(&mut buffer, depth + 1, out);
                }
                "script" | "style" => {
                    let text: String = node.children().filter_map(|c| c.value().as_text().map(|t| t.to_string())).collect();
                    let pad = "  ".repeat(depth + 1);
                    out.extend(text.lines().map(str::trim).filter(|l| !l.is_empty()).map(|l| format!("{pad}|{l}")));
                }
                _ => node.children().for_each(|c| walk(c, depth + 1, out)),
            }
        }
    }
}

/// Recognizes the known code block shapes; returns the language and the `<pre>` holding the code.
fn code_block(node: NodeRef<Node>) -> Option<(String, NodeRef<Node>)> {
    let el = node.value().as_element()?;
    let has_class = |name: &str| el.classes().any(|c| c == name);
    let pre = match el.name() {
        "pre" => node,
        "div" if has_class("highlighter-rouge") => find_pre(node)?,
        "figure" if has_class("highlight") => find_pre(node)?,
        _ => return None,
    };
    let code = pre.children().find(|c| c.value().as_element().is_some_and(|e| e.name() == "code"))?;
    // A <pre> without a <code> child is not a code block (and keeps the generic handling).
    let language = [node, pre, code]
        .iter()
        .filter_map(|n| n.value().as_element())
        .flat_map(|e| e.classes())
        .find_map(|c| c.strip_prefix("language-"))
        .unwrap_or("plaintext")
        .to_string();
    Some((language, pre))
}

fn find_pre(node: NodeRef<Node>) -> Option<NodeRef<Node>> {
    node.descendants().find(|n| n.value().as_element().is_some_and(|e| e.name() == "pre"))
}

/// Inside `<pre>`: text is verbatim, `<span>`s are transparent, other elements keep their structure.
fn walk_pre(node: NodeRef<Node>, depth: usize, buffer: &mut String, out: &mut Vec<String>) {
    match node.value() {
        Node::Text(t) => buffer.push_str(t),
        Node::Element(el) if el.name() == "span" => node.children().for_each(|c| walk_pre(c, depth, buffer, out)),
        Node::Element(el) => {
            flush_verbatim(buffer, depth, out);
            out.push(format!("{}{}", "  ".repeat(depth), open_tag(el)));
            node.children().for_each(|c| walk_pre(c, depth + 1, buffer, out));
            flush_verbatim(buffer, depth + 1, out);
        }
        _ => {}
    }
}

fn flush_verbatim(buffer: &mut String, depth: usize, out: &mut Vec<String>) {
    if buffer.is_empty() {
        return;
    }
    let pad = "  ".repeat(depth);
    out.extend(buffer.split('\n').map(|l| format!("{pad}|{l}")));
    buffer.clear();
}

fn open_tag(el: &scraper::node::Element) -> String {
    let mut attrs: Vec<(&str, String)> = el
        .attrs()
        .map(|(name, value)| {
            let value = if name == "class" {
                let mut tokens: Vec<&str> = value.split_whitespace().collect();
                tokens.sort_unstable();
                tokens.join(" ")
            } else {
                value.to_string()
            };
            (name, value)
        })
        .collect();
    attrs.sort();
    let mut tag = format!("<{}", el.name());
    for (name, value) in attrs {
        tag.push_str(&format!(" {name}={value:?}"));
    }
    tag.push('>');
    tag
}

fn collapse(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_space = false;
    for ch in text.chars() {
        if ch.is_ascii_whitespace() {
            if !in_space {
                out.push(' ');
            }
            in_space = true;
        } else {
            out.push(ch);
            in_space = false;
        }
    }
    out
}

/// A space at the edge of a text node only matters when the neighbour is an inline element.
fn edge_trimmed<'a>(node: NodeRef<Node>, text: &'a str) -> &'a str {
    let inline = |n: Option<NodeRef<Node>>| {
        n.and_then(|n| n.value().as_element().map(|e| INLINE.contains(&e.name()))).unwrap_or(false)
    };
    let text = if inline(node.prev_sibling()) { text } else { text.trim_start_matches(' ') };
    if inline(node.next_sibling()) { text } else { text.trim_end_matches(' ') }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(html: &str) -> Vec<String> {
        fragment_lines(html, "x.html", Side::Both, &AllowList::default())
    }

    #[test]
    fn code_block_shapes_are_equivalent() {
        let rouge = r#"<div class="language-rust highlighter-rouge"><div class="highlight"><pre class="highlight"><code><span class="k">let</span> a = 1;
</code></pre></div></div>"#;
        let jekyll = r#"<figure class="highlight"><pre><code class="language-rust" data-lang="rust"><span class="k">let</span> a = 1;</code></pre></figure>"#;
        let plain = "<pre class=\"highlight s-code\"><code class=\"language-rust\"><span class=\"s-source\">let a = 1;\n</span></code></pre>";
        assert_eq!(lines(rouge), vec!["<codeblock language=\"rust\">", "  |let a = 1;"]);
        assert_eq!(lines(rouge), lines(jekyll));
        assert_eq!(lines(rouge), lines(plain));
    }

    #[test]
    fn code_block_language_and_text_still_matter() {
        let block = |language: &str, code: &str| lines(&format!("<pre><code class=\"language-{language}\">{code}</code></pre>"));
        assert_ne!(block("rust", "let a = 1;"), block("toml", "let a = 1;"));
        assert_ne!(block("rust", "let a = 1;"), block("rust", "let a = 2;"));
        assert_ne!(block("rust", "a\n\nb"), block("rust", "a\nb"));
        assert_eq!(lines("<pre><code>x</code></pre>"), lines("<pre><code class=\"language-plaintext\">x\n</code></pre>"));
    }
}
