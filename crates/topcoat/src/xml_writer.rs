//! RSS / sitemap で共通して使う最小限の XML エスケープ・要素書き出しヘルパー。
//!
//! 外部の XML シリアライズ crate には依存せず、標準ライブラリ (`std::fmt::Write`) のみで
//! 実装する。

use std::fmt::Write as _;

/// XML のテキストノードとして安全な文字列にエスケープする。
///
/// `&`, `<`, `>` をエスケープする。`"` / `'` はテキストノードでは必須ではないため
/// エスケープしない (属性値には [`escape_attr`] を使うこと)。
pub fn escape_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// XML の属性値として安全な文字列にエスケープする。
///
/// `&`, `<`, `>`, `"`, `'` をすべてエスケープする。
pub fn escape_attr(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// `<tag>escaped(content)</tag>` を改行付きで `out` に書き出す。
pub fn write_text_element(out: &mut String, tag: &str, content: &str) {
    let _ = writeln!(out, "<{tag}>{}</{tag}>", escape_text(content));
}

/// `<tag attr="escaped(attr_value)">escaped(content)</tag>` を改行付きで `out` に書き出す。
pub fn write_text_element_with_attr(
    out: &mut String,
    tag: &str,
    attr_name: &str,
    attr_value: &str,
    content: &str,
) {
    let _ = writeln!(
        out,
        "<{tag} {attr_name}=\"{}\">{}</{tag}>",
        escape_attr(attr_value),
        escape_text(content)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_text_escapes_amp_lt_gt() {
        assert_eq!(escape_text("a & b < c > d"), "a &amp; b &lt; c &gt; d");
    }

    #[test]
    fn escape_text_leaves_quotes_untouched() {
        assert_eq!(escape_text(r#"say "hi" 'bye'"#), r#"say "hi" 'bye'"#);
    }

    #[test]
    fn escape_attr_escapes_all_five_chars() {
        assert_eq!(
            escape_attr(r#"a & b < c > d " e ' f"#),
            "a &amp; b &lt; c &gt; d &quot; e &apos; f"
        );
    }

    #[test]
    fn write_text_element_wraps_and_escapes() {
        let mut out = String::new();
        write_text_element(&mut out, "title", "A & B");
        assert_eq!(out, "<title>A &amp; B</title>\n");
    }

    #[test]
    fn write_text_element_with_attr_escapes_both_parts() {
        let mut out = String::new();
        write_text_element_with_attr(
            &mut out,
            "guid",
            "isPermaLink",
            "true",
            "https://x/?a=1&b=2",
        );
        assert_eq!(
            out,
            "<guid isPermaLink=\"true\">https://x/?a=1&amp;b=2</guid>\n"
        );
    }
}
