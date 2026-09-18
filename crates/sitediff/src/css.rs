//! CSS → canonical line form, used only when the files are not byte-identical.
//!
//! Normalizes formatting differences between Sass implementations: comments, whitespace,
//! the optional last `;` in a block, hex color case. One line per rule / at-rule boundary.

pub fn lines(source: &str) -> Vec<String> {
    let source = strip_comments(source);
    let mut out = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut in_string: Option<char> = None;

    let flush = |current: &mut String, depth: usize, out: &mut Vec<String>| {
        let line = tidy(current);
        if !line.is_empty() {
            out.push(format!("{}{line}", "  ".repeat(depth)));
        }
        current.clear();
    };

    for ch in source.chars() {
        if let Some(quote) = in_string {
            current.push(ch);
            if ch == quote {
                in_string = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => {
                in_string = Some(ch);
                current.push(ch);
            }
            '{' => {
                current.push('{');
                flush(&mut current, depth, &mut out);
                depth += 1;
            }
            '}' => {
                flush(&mut current, depth, &mut out);
                depth = depth.saturating_sub(1);
                out.push(format!("{}}}", "  ".repeat(depth)));
            }
            ';' => flush(&mut current, depth, &mut out),
            _ => current.push(ch),
        }
    }
    flush(&mut current, depth, &mut out);
    out
}

fn strip_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = rest[start + 2..].split_once("*/").map_or("", |(_, after)| after);
    }
    out.push_str(rest);
    out
}

/// Collapse whitespace, remove it around punctuation, lowercase hex colors.
fn tidy(piece: &str) -> String {
    let collapsed = piece.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = String::with_capacity(collapsed.len());
    let chars: Vec<char> = collapsed.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        let tight = |c: char| matches!(c, '{' | ':' | ',' | '>' | '+' | '~' | '(' | ')');
        if ch == ' ' {
            let prev = if i > 0 { chars[i - 1] } else { ' ' };
            let next = chars.get(i + 1).copied().unwrap_or(' ');
            // keep the space in selectors like `a :hover`? Sass never emits that; treat `:` as tight.
            if tight(prev) && prev != ')' || tight(next) && next != '(' {
                continue;
            }
        }
        out.push(ch);
    }
    lowercase_hex(&out)
}

fn lowercase_hex(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        out.push(ch);
        if ch == '#' {
            while let Some(&next) = chars.peek().filter(|c| c.is_ascii_hexdigit()) {
                out.push(next.to_ascii_lowercase());
                chars.next();
            }
        }
    }
    out
}
