//! Build-time syntax highlighting: code becomes `<span class="s-…">` tokens, colored by a
//! stylesheet generated from the configured theme. No JavaScript involved.

use anyhow::{Context, Result, anyhow};
use syntect::highlighting::Theme;
use syntect::html::{ClassStyle, ClassedHTMLGenerator, css_for_theme_with_class_style};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

/// Token classes are prefixed so they cannot collide with the site's own class names.
const CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed { prefix: "s-" };

pub struct Highlighter {
    syntaxes: SyntaxSet,
    theme: Theme,
}

impl Highlighter {
    /// `theme` is one of two-face's embedded theme names, e.g. `GitHub` or `OneHalfLight`.
    pub fn new(theme: &str) -> Result<Self> {
        let themes = two_face::theme::extra();
        let names = two_face::theme::EmbeddedLazyThemeSet::theme_names();
        let name = names.iter().copied().find(|n| n.as_name().eq_ignore_ascii_case(theme)).with_context(|| {
            let known: Vec<&str> = names.iter().map(|n| n.as_name()).collect();
            format!("unknown syntax_theme {theme:?}; available: {}", known.join(", "))
        })?;
        Ok(Highlighter { syntaxes: two_face::syntax::extra_newlines(), theme: themes.get(name).clone() })
    }

    /// The stylesheet that gives the token classes their colors.
    pub fn css(&self) -> Result<String> {
        css_for_theme_with_class_style(&self.theme, CLASS_STYLE).map_err(|e| anyhow!("generating syntax css: {e}"))
    }

    /// `<pre class="highlight s-code"><code class="language-…">tokens</code></pre>`
    pub fn code_block(&self, language: &str, code: &str) -> Result<String> {
        let syntax = self.syntax_for(language);
        let mut generator = ClassedHTMLGenerator::new_with_class_style(syntax, &self.syntaxes, CLASS_STYLE);
        for line in LinesWithEndings::from(code) {
            generator.parse_html_for_line_which_includes_newline(line).map_err(|e| anyhow!("highlighting {language}: {e}"))?;
        }
        let label = if language.is_empty() { "plaintext" } else { language };
        Ok(format!("<pre class=\"highlight s-code\"><code class=\"language-{label}\">{}</code></pre>\n", generator.finalize()))
    }

    /// The same block without tokens or styling hooks, for the feed.
    pub fn plain_code_block(language: &str, code: &str) -> String {
        let label = if language.is_empty() { "plaintext" } else { language };
        let mut escaped = String::with_capacity(code.len());
        for ch in code.chars() {
            match ch {
                '&' => escaped.push_str("&amp;"),
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                _ => escaped.push(ch),
            }
        }
        format!("<pre><code class=\"language-{label}\">{escaped}</code></pre>\n")
    }

    /// Fence info strings used on this blog that are not syntect tokens; anything unknown is plain text.
    fn syntax_for(&self, language: &str) -> &SyntaxReference {
        let token = match language {
            "c++" => "cpp",
            "C" => "c",
            "console" | "txt" | "plain" | "plaintext" | "text" | "cfg" | "gdb" | "" => return self.syntaxes.find_syntax_plain_text(),
            other => other,
        };
        self.syntaxes.find_syntax_by_token(token).unwrap_or_else(|| self.syntaxes.find_syntax_plain_text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_used_on_the_blog_has_a_grammar() {
        let highlighter = Highlighter::new("GitHub").unwrap();
        for language in ["rust", "toml", "cpp", "c++", "c", "C", "bash", "glsl", "cs", "haskell", "python"] {
            let syntax = highlighter.syntax_for(language);
            assert_ne!(syntax.name, "Plain Text", "{language} fell back to plain text");
        }
    }

    #[test]
    fn code_text_survives_highlighting() {
        let highlighter = Highlighter::new("github").unwrap();
        let html = highlighter.code_block("rust", "fn main() {\n    let a = 1 < 2 && true;\n}\n").unwrap();
        assert!(html.starts_with("<pre class=\"highlight s-code\"><code class=\"language-rust\">"));
        assert!(html.contains("<span class=\"s-"));
        let text: String = {
            let mut out = String::new();
            let mut in_tag = false;
            for ch in html.chars() {
                match ch {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    _ if !in_tag => out.push(ch),
                    _ => {}
                }
            }
            out
        };
        assert_eq!(text, "fn main() {\n    let a = 1 &lt; 2 &amp;&amp; true;\n}\n\n");
    }

    #[test]
    fn unknown_theme_lists_alternatives() {
        let error = Highlighter::new("nope").err().unwrap().to_string();
        assert!(error.contains("GitHub"), "{error}");
    }
}
