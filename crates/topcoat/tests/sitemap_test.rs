//! sitemap.xml 生成の統合テスト。
//!
//! トップページ・作品詳細・タグ一覧・シリーズ一覧の全 URL が漏れなく含まれることを、
//! crate 公開 API (`topcoat::build::sitemap`) を通して検証する。

use topcoat::build::sitemap::{
    generate_sitemap, SitemapInput, SitemapSeriesEntry, SitemapTagEntry, SitemapWorkEntry,
};

fn empty_input() -> SitemapInput {
    SitemapInput {
        base_url: "https://example.com".to_string(),
        works: vec![],
        tags: vec![],
        series: vec![],
    }
}

#[test]
fn zero_entries_still_includes_top_page_and_is_well_formed() {
    let xml = generate_sitemap(&empty_input());
    assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(xml.contains("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"));
    assert!(xml.contains("</urlset>"));
    assert_eq!(xml.matches("<url>").count(), 1);
    assert!(xml.contains("<loc>https://example.com/</loc>"));
}

#[test]
fn one_work_one_tag_one_series_are_all_present() {
    let input = SitemapInput {
        base_url: "https://example.com".to_string(),
        works: vec![SitemapWorkEntry {
            slug: "work-1".to_string(),
            lastmod: Some("2026-01-01T00:00:00Z".to_string()),
        }],
        tags: vec![SitemapTagEntry {
            slug: "tag-1".to_string(),
        }],
        series: vec![SitemapSeriesEntry {
            slug: "series-1".to_string(),
        }],
    };
    let xml = generate_sitemap(&input);
    // top page + work + tag + series = 4件
    assert_eq!(xml.matches("<url>").count(), 4);
    assert!(xml.contains("<loc>https://example.com/</loc>"));
    assert!(xml.contains("<loc>https://example.com/works/work-1/</loc>"));
    assert!(xml.contains("<loc>https://example.com/tags/tag-1/</loc>"));
    assert!(xml.contains("<loc>https://example.com/series/series-1/</loc>"));
}

#[test]
fn multiple_works_tags_series_are_all_enumerated_without_omission() {
    let input = SitemapInput {
        base_url: "https://example.com".to_string(),
        works: (1..=3)
            .map(|i| SitemapWorkEntry {
                slug: format!("work-{i}"),
                lastmod: None,
            })
            .collect(),
        tags: (1..=2)
            .map(|i| SitemapTagEntry {
                slug: format!("tag-{i}"),
            })
            .collect(),
        series: (1..=2)
            .map(|i| SitemapSeriesEntry {
                slug: format!("series-{i}"),
            })
            .collect(),
    };
    let xml = generate_sitemap(&input);
    // top page 1 + works 3 + tags 2 + series 2 = 8件
    assert_eq!(xml.matches("<url>").count(), 8);
    for i in 1..=3 {
        assert!(xml.contains(format!("<loc>https://example.com/works/work-{i}/</loc>").as_str()));
    }
    for i in 1..=2 {
        assert!(xml.contains(format!("<loc>https://example.com/tags/tag-{i}/</loc>").as_str()));
        assert!(xml.contains(format!("<loc>https://example.com/series/series-{i}/</loc>").as_str()));
    }
}

#[test]
fn special_characters_in_slug_are_escaped() {
    let input = SitemapInput {
        base_url: "https://example.com".to_string(),
        works: vec![SitemapWorkEntry {
            slug: "a&b<c>d".to_string(),
            lastmod: None,
        }],
        tags: vec![],
        series: vec![],
    };
    let xml = generate_sitemap(&input);
    assert!(xml.contains("<loc>https://example.com/works/a&amp;b&lt;c&gt;d/</loc>"));
    assert!(!xml.contains("a&b<c>d/</loc>"));
}

#[test]
fn lastmod_present_only_when_supplied() {
    let input = SitemapInput {
        base_url: "https://example.com".to_string(),
        works: vec![
            SitemapWorkEntry {
                slug: "with-lastmod".to_string(),
                lastmod: Some("2026-05-01T00:00:00Z".to_string()),
            },
            SitemapWorkEntry {
                slug: "without-lastmod".to_string(),
                lastmod: None,
            },
        ],
        tags: vec![],
        series: vec![],
    };
    let xml = generate_sitemap(&input);
    assert_eq!(xml.matches("<lastmod>").count(), 1);
    assert!(xml.contains("<lastmod>2026-05-01T00:00:00Z</lastmod>"));
}
