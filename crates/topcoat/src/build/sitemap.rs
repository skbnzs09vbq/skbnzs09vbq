//! `sitemap.xml` 生成。
//!
//! トップページ・作品詳細ページ (`/works/<slug>/`)・タグ一覧ページ (`/tags/<slug>/`)・
//! シリーズ一覧ページ (`/series/<slug>/`) の全 URL を列挙し、`<urlset>`/`<url>`/`<loc>`/
//! `<lastmod>` からなる sitemap.xml を [`crate::xml_writer`] を使った手書き XML
//! シリアライズで生成する。外部の XML/サイトマップ生成 crate には依存しない。
//!
//! この関数は純粋関数であり、DB アクセスは行わない。

use crate::xml_writer::write_text_element;

/// サイトマップに載せる作品ページ 1件分の情報。
pub struct SitemapWorkEntry {
    pub slug: String,
    /// RFC3339 (UTC) 形式。無ければ `<lastmod>` を出力しない。
    pub lastmod: Option<String>,
}

/// サイトマップに載せるタグ一覧ページ 1件分の情報。
pub struct SitemapTagEntry {
    pub slug: String,
}

/// サイトマップに載せるシリーズ一覧ページ 1件分の情報。
pub struct SitemapSeriesEntry {
    pub slug: String,
}

/// sitemap.xml 生成への入力。
pub struct SitemapInput {
    /// 末尾スラッシュなしのベース URL (例: `https://example.com`)
    pub base_url: String,
    pub works: Vec<SitemapWorkEntry>,
    pub tags: Vec<SitemapTagEntry>,
    pub series: Vec<SitemapSeriesEntry>,
}

struct UrlEntry {
    loc: String,
    lastmod: Option<String>,
}

/// トップページ・作品詳細・タグ一覧・シリーズ一覧の全 URL を含む sitemap.xml 文字列を生成する。
pub fn generate_sitemap(input: &SitemapInput) -> String {
    let base = input.base_url.trim_end_matches('/');

    let mut urls =
        Vec::with_capacity(1 + input.works.len() + input.tags.len() + input.series.len());

    // トップページ
    urls.push(UrlEntry {
        loc: format!("{base}/"),
        lastmod: None,
    });

    for work in &input.works {
        urls.push(UrlEntry {
            loc: format!("{base}/works/{}/", work.slug),
            lastmod: work.lastmod.clone(),
        });
    }

    for tag in &input.tags {
        urls.push(UrlEntry {
            loc: format!("{base}/tags/{}/", tag.slug),
            lastmod: None,
        });
    }

    for series in &input.series {
        urls.push(UrlEntry {
            loc: format!("{base}/series/{}/", series.slug),
            lastmod: None,
        });
    }

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    for url in &urls {
        out.push_str("<url>\n");
        write_text_element(&mut out, "loc", &url.loc);
        if let Some(lastmod) = &url.lastmod {
            write_text_element(&mut out, "lastmod", lastmod);
        }
        out.push_str("</url>\n");
    }

    out.push_str("</urlset>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_input() -> SitemapInput {
        SitemapInput {
            base_url: "https://example.com".to_string(),
            works: vec![],
            tags: vec![],
            series: vec![],
        }
    }

    #[test]
    fn includes_top_page_even_with_no_data() {
        let xml = generate_sitemap(&empty_input());
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"));
        assert!(xml.contains("<loc>https://example.com/</loc>\n"));
    }

    #[test]
    fn includes_single_work_url() {
        let mut input = empty_input();
        input.works.push(SitemapWorkEntry {
            slug: "work-1".into(),
            lastmod: Some("2026-01-01T00:00:00Z".into()),
        });
        let xml = generate_sitemap(&input);
        assert!(xml.contains("<loc>https://example.com/works/work-1/</loc>\n"));
        assert!(xml.contains("<lastmod>2026-01-01T00:00:00Z</lastmod>\n"));
    }

    #[test]
    fn includes_all_work_tag_series_urls() {
        let mut input = empty_input();
        input.works.push(SitemapWorkEntry {
            slug: "work-1".into(),
            lastmod: Some("2026-01-01T00:00:00Z".into()),
        });
        input.works.push(SitemapWorkEntry {
            slug: "work-2".into(),
            lastmod: None,
        });
        input.tags.push(SitemapTagEntry {
            slug: "tag-a".into(),
        });
        input.series.push(SitemapSeriesEntry {
            slug: "series-a".into(),
        });

        let xml = generate_sitemap(&input);
        assert!(xml.contains("<loc>https://example.com/works/work-1/</loc>\n"));
        assert!(xml.contains("<loc>https://example.com/works/work-2/</loc>\n"));
        assert!(xml.contains("<loc>https://example.com/tags/tag-a/</loc>\n"));
        assert!(xml.contains("<loc>https://example.com/series/series-a/</loc>\n"));
    }

    #[test]
    fn work_without_lastmod_omits_lastmod_tag() {
        let mut input = empty_input();
        input.works.push(SitemapWorkEntry {
            slug: "work-2".into(),
            lastmod: None,
        });
        let xml = generate_sitemap(&input);
        let start = xml.find("works/work-2/").unwrap();
        let block_start = xml[..start].rfind("<url>").unwrap();
        let block_end = xml[start..].find("</url>").unwrap() + start;
        let block = &xml[block_start..block_end];
        assert!(!block.contains("<lastmod>"));
    }

    #[test]
    fn escapes_special_characters_in_slug() {
        let mut input = empty_input();
        input.works.push(SitemapWorkEntry {
            slug: "a&b".into(),
            lastmod: None,
        });
        let xml = generate_sitemap(&input);
        assert!(xml.contains("<loc>https://example.com/works/a&amp;b/</loc>\n"));
    }

    #[test]
    fn strips_trailing_slash_from_base_url() {
        let mut input = empty_input();
        input.base_url = "https://example.com/".to_string();
        let xml = generate_sitemap(&input);
        assert!(xml.contains("<loc>https://example.com/</loc>\n"));
        assert!(!xml.contains("https://example.com//"));
    }
}
