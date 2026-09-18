//! CSS → canonical line form, used only when the files are not byte-identical.
//!
//! Normalizes notation differences between Sass implementations: BOM, comments, whitespace, the
//! optional last `;` in a block, hex color case, leading zeros (`0.5em` / `.5em`), escapes
//! (`"\201C"` / `"“"`) and the order of selectors in a selector list. One line per rule boundary.
//! Not normalized: equivalent values spelled differently (`black` / `#000`) — allowlist those.

pub fn lines(source: &str) -> Vec<String> {
    let source = strip_comments(source.trim_start_matches('\u{feff}'));
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
                current = sorted_selectors(&current);
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

/// Collapse whitespace, remove it around punctuation, then normalize notation.
fn tidy(piece: &str) -> String {
    let collapsed = piece.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = String::with_capacity(collapsed.len());
    let chars: Vec<char> = collapsed.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        let tight = |c: char| matches!(c, '{' | ':' | ',' | '>' | '+' | '~' | '(' | ')');
        if ch == ' ' {
            let prev = if i > 0 { chars[i - 1] } else { ' ' };
            let next = chars.get(i + 1).copied().unwrap_or(' ');
            if tight(prev) && prev != ')' || tight(next) && next != '(' {
                continue;
            }
        }
        out.push(ch);
    }
    strip_leading_zeros(&decode_escapes(&lowercase_hex(&out)))
}

/// `a, b` and `b, a` select the same elements. At-rule preludes are left alone.
fn sorted_selectors(prelude: &str) -> String {
    if prelude.trim_start().starts_with('@') {
        return prelude.to_string();
    }
    let mut selectors: Vec<&str> = prelude.split(',').map(str::trim).collect();
    selectors.sort_unstable();
    selectors.join(",")
}

fn lowercase_hex(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        out.push(ch);
        if ch == '#' {
            while let Some(next) = chars.next_if(char::is_ascii_hexdigit) {
                out.push(next.to_ascii_lowercase());
            }
        }
    }
    out
}

/// `0.5em` → `.5em`
fn strip_leading_zeros(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (i, &ch) in chars.iter().enumerate() {
        let starts_number = i == 0 || !(chars[i - 1].is_ascii_alphanumeric() || chars[i - 1] == '.');
        let before_fraction = chars.get(i + 1) == Some(&'.') && chars.get(i + 2).is_some_and(char::is_ascii_digit);
        if ch == '0' && starts_number && before_fraction {
            continue;
        }
        out.push(ch);
    }
    out
}

/// `\201C` (1-6 hex digits, optionally followed by one space) → the character itself.
fn decode_escapes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let mut hex = String::new();
        while hex.len() < 6
            && let Some(digit) = chars.next_if(char::is_ascii_hexdigit)
        {
            hex.push(digit);
        }
        match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
            Some(decoded) => {
                out.push(decoded);
                chars.next_if_eq(&' ');
            }
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::lines;

    #[test]
    fn notation_differences_are_normalized() {
        let ruby = r#".b:after,.a:after{content:"\201C";padding:0.25em 40px;opacity:0.6;color:#FFF}"#;
        let grass = "\u{feff}.a:after, .b:after {\n  content: \"“\";\n  padding: .25em 40px;\n  opacity: .6;\n  color: #fff;\n}\n";
        assert_eq!(lines(ruby), lines(grass));
    }

    #[test]
    fn values_are_not_normalized() {
        assert_ne!(lines("a{width:10.5px}"), lines("a{width:1.5px}"));
        assert_ne!(lines("a{width:100.5px}"), lines("a{width:10.5px}"));
        assert_ne!(lines("a{color:black}"), lines("a{color:#000}"));
        assert_ne!(lines("@media screen and (max-width:600px){a{b:c}}"), lines("@media screen and (max-width:800px){a{b:c}}"));
    }
}
