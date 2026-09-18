//! XML feed → canonical line form.
//!
//! Every leaf becomes `path/to/element: "text"`, so allowlist line rules can target e.g.
//! `rss/channel/lastBuildDate` without also matching `rss/channel/item[3]/pubDate`.
//! `<description>` holds escaped HTML: it is unescaped and compared as normalized HTML.

use anyhow::Result;
use roxmltree::{Document, Node};

use crate::allow::{AllowList, Side};
use crate::html;

pub fn lines(source: &str, path: &str, side: Side, allow: &AllowList) -> Result<Vec<String>> {
    let doc = Document::parse(source)?;
    let mut out = Vec::new();
    let root = doc.root_element();
    walk(root, &name_of(root), path, side, allow, &mut out);
    Ok(out)
}

fn walk(node: Node, at: &str, path: &str, side: Side, allow: &AllowList, out: &mut Vec<String>) {
    let mut attrs: Vec<_> = node.attributes().map(|a| (a.name(), a.value())).collect();
    attrs.sort();
    for (name, value) in attrs {
        out.push(format!("{at}@{name}: {value:?}"));
    }

    let children: Vec<Node> = node.children().filter(Node::is_element).collect();
    if children.is_empty() {
        let text = node.text().unwrap_or("");
        if node.tag_name().name() == "description" {
            out.push(format!("{at}:"));
            out.extend(html::fragment_lines(text, path, side, allow).into_iter().map(|l| format!("  {l}")));
        } else {
            out.push(format!("{at}: {:?}", text.split_whitespace().collect::<Vec<_>>().join(" ")));
        }
        return;
    }
    for (i, child) in children.iter().enumerate() {
        let name = name_of(*child);
        let same: Vec<usize> = (0..children.len()).filter(|&j| name_of(children[j]) == name).collect();
        let at = if same.len() > 1 {
            format!("{at}/{name}[{}]", same.iter().position(|&j| j == i).unwrap() + 1)
        } else {
            format!("{at}/{name}")
        };
        walk(*child, &at, path, side, allow, out);
    }
}

fn name_of(node: Node) -> String {
    let tag = node.tag_name();
    match tag.namespace().and_then(|ns| node.lookup_prefix(ns)) {
        Some(prefix) if !prefix.is_empty() => format!("{prefix}:{}", tag.name()),
        _ => tag.name().to_string(),
    }
}
