//! Jekyll filters used by the templates that the `liquid` crate does not ship.

use liquid_core::{Display_filter, Filter, FilterReflection, ParseFilter, Result, Runtime, Value, ValueView};

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(name = "xml_escape", description = "Escapes text for use in XML.", parsed(XmlEscapeFilter))]
pub struct XmlEscape;

#[derive(Debug, Default, Display_filter)]
#[name = "xml_escape"]
struct XmlEscapeFilter;

impl Filter for XmlEscapeFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
        if input.is_nil() {
            return Ok(Value::Nil);
        }
        Ok(Value::scalar(xml_escape(&input.to_kstr())))
    }
}

/// Same replacements as Ruby's `String#encode(xml: :attr)` minus the surrounding quotes, which is
/// what Jekyll's `xml_escape` does.
pub fn xml_escape(text: &str) -> String {
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

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(name = "date_to_rfc822", description = "Formats a date as RFC 822.", parsed(DateToRfc822Filter))]
pub struct DateToRfc822;

#[derive(Debug, Default, Display_filter)]
#[name = "date_to_rfc822"]
struct DateToRfc822Filter;

impl Filter for DateToRfc822Filter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
        match input.as_scalar().and_then(|s| s.to_date_time()) {
            Some(date) => Ok(Value::scalar(date.to_rfc2822())),
            None => Ok(input.to_value()),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn xml_escape_matches_jekyll() {
        assert_eq!(super::xml_escape(r#"<a href="x">Tom & Jerry's</a>"#), "&lt;a href=&quot;x&quot;&gt;Tom &amp; Jerry's&lt;/a&gt;");
    }
}
