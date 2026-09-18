//! Markdown → HTML.
//!
//! Plain CommonMark for now; kramdown-compatible output (heading ids, typography, code block
//! wrappers, `{% highlight %}`) is MIGRATION Phase 5.

use pulldown_cmark::{Options, Parser, html};

pub fn render(source: &str) -> String {
    let parser = Parser::new_ext(source, Options::ENABLE_TABLES);
    let mut out = String::with_capacity(source.len() * 2);
    html::push_html(&mut out, parser);
    out
}
