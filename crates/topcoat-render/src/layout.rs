//! 全ページ共通のベースレイアウト (`templates/layout.html`) の Askama テンプレート。
//!
//! header/nav/footer/OGP メタタグを含む HTML の骨格を定義し、各ページ固有のテンプレートは
//! `{% extends "layout.html" %}` + `{% block content %}` でこのレイアウトを継承する想定
//! (issue #30: フロントエンド基盤)。
//!
//! 現時点では既存ページ (index/work_detail/tags/series) は `tera` ベースの実装のままであり、
//! 本レイアウトはそれらを置き換えるものではない。以降のフロントエンド系 issue が、
//! このレイアウトを継承する Askama テンプレートへ順次移行していく土台として導入する。

use askama::Template;

/// `templates/layout.html` をレンダリングするための、ベースレイアウトが必要とする変数一式。
///
/// このレイアウト単体でも (中身が空の `content` ブロックのまま) レンダリング可能であり、
/// `{% extends "layout.html" %}` する子テンプレートは、自身の `#[derive(Template)]` 構造体に
/// 同じフィールドを含めることでこれらの変数を継承する。
#[derive(Debug, Clone, Template)]
#[template(path = "layout.html")]
pub struct BaseLayout<'a> {
    pub site_title: &'a str,
    pub site_description: &'a str,
    /// 末尾スラッシュなしのベース URL (例: `https://example.com`)
    pub base_url: &'a str,
    pub current_year: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> BaseLayout<'static> {
        BaseLayout {
            site_title: "topcoat",
            site_description: "topcoat works feed",
            base_url: "https://example.com",
            current_year: 2026,
        }
    }

    #[test]
    fn renders_doctype_and_dark_theme_by_default() {
        let html = layout().render().expect("layout should render");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains(r#"<html lang="ja" data-theme="dark">"#));
    }

    #[test]
    fn renders_title_and_description() {
        let html = layout().render().expect("layout should render");
        assert!(html.contains("<title>topcoat</title>"));
        assert!(html.contains(r#"<meta name="description" content="topcoat works feed">"#));
    }

    #[test]
    fn renders_ogp_meta_tags() {
        let html = layout().render().expect("layout should render");
        assert!(html.contains(r#"property="og:title" content="topcoat""#));
        assert!(html.contains(r#"property="og:description" content="topcoat works feed""#));
        assert!(html.contains(r#"property="og:url" content="https://example.com/""#));
        assert!(html.contains(r#"name="twitter:card" content="summary_large_image""#));
    }

    #[test]
    fn renders_nav_and_footer() {
        let html = layout().render().expect("layout should render");
        assert!(html.contains(r#"class="site-nav""#));
        assert!(html.contains(r#"href="/tags/""#));
        assert!(html.contains(r#"href="/series/""#));
        assert!(html.contains(r#"class="site-footer""#));
        assert!(html.contains("2026"));
    }

    #[test]
    fn links_design_token_stylesheet() {
        let html = layout().render().expect("layout should render");
        assert!(html.contains(r#"<link rel="stylesheet" href="/assets/css/tokens.css">"#));
    }
}
