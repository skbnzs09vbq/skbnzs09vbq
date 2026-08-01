//! タグ別一覧ページ (`dist/tags/<slug>/index.html`) 生成の統合テスト。
//!
//! 0件・1件・複数タグ・特殊文字を含む title 等のケースを、crate 公開 API
//! (`topcoat::build::tags` / `topcoat::build::write_tag_pages`) を通して検証する。

use std::fs;

use topcoat::build::tags::{render_tag_page, write_tag_pages, TagPageEntry, TagPageWork};
use topcoat::build::{write_tag_pages as write_tag_pages_from_site_data, SiteData};
use topcoat::models::{Tag, Work};

fn entry(slug: &str, name: &str, works: Vec<TagPageWork>) -> TagPageEntry {
    TagPageEntry {
        slug: slug.to_string(),
        name: name.to_string(),
        works,
    }
}

#[test]
fn zero_works_renders_well_formed_page_with_no_items() {
    let html = render_tag_page(&entry("illustration", "イラスト", vec![])).unwrap();
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<h1>イラスト</h1>"));
    assert!(!html.contains("work-item"));
}

#[test]
fn one_work_includes_title_link_and_thumbnail() {
    let html = render_tag_page(&entry(
        "illustration",
        "イラスト",
        vec![TagPageWork {
            slug: "work-1".to_string(),
            title: "作品タイトル".to_string(),
            thumbnail: Some("/thumbs/work-1.png".to_string()),
        }],
    ))
    .unwrap();
    assert!(html.contains(r#"<a class="work-link" href="/works/work-1/">作品タイトル</a>"#));
    assert!(html.contains(r#"src="/thumbs/work-1.png""#));
}

#[test]
fn escapes_special_characters_in_tag_name_and_work_title() {
    let html = render_tag_page(&entry(
        "special",
        "<script>&\"tag\"",
        vec![TagPageWork {
            slug: "work-1".to_string(),
            title: "<script>&\"title\"".to_string(),
            thumbnail: None,
        }],
    ))
    .unwrap();
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;&amp;&quot;tag&quot;"));
    assert!(html.contains("&lt;script&gt;&amp;&quot;title&quot;"));
}

#[test]
fn write_tag_pages_writes_dist_tags_slug_index_html_for_each_tag() {
    let dir = tempfile::tempdir().unwrap();
    let entries = vec![
        entry(
            "illustration",
            "イラスト",
            vec![TagPageWork {
                slug: "work-1".to_string(),
                title: "作品1".to_string(),
                thumbnail: None,
            }],
        ),
        entry(
            "manga",
            "漫画",
            vec![TagPageWork {
                slug: "work-2".to_string(),
                title: "作品2".to_string(),
                thumbnail: None,
            }],
        ),
    ];

    write_tag_pages(dir.path(), &entries).unwrap();

    let illustration = fs::read_to_string(dir.path().join("tags/illustration/index.html"))
        .expect("dist/tags/illustration/index.html が書き出されていること");
    assert!(illustration.contains("作品1"));
    assert!(!illustration.contains("作品2"));

    let manga = fs::read_to_string(dir.path().join("tags/manga/index.html"))
        .expect("dist/tags/manga/index.html が書き出されていること");
    assert!(manga.contains("作品2"));
    assert!(!manga.contains("作品1"));
}

#[test]
fn site_data_tag_pages_only_include_works_tagged_with_that_tag() {
    let dir = tempfile::tempdir().unwrap();

    let data = SiteData {
        works: vec![
            Work {
                slug: "work-1".to_string(),
                title: "作品1".to_string(),
                description: "".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: None,
                tags: vec!["illustration".to_string()],
                thumbnail: None,
            },
            Work {
                slug: "work-2".to_string(),
                title: "作品2".to_string(),
                description: "".to_string(),
                created_at: "2026-01-02T00:00:00Z".to_string(),
                updated_at: None,
                tags: vec!["illustration".to_string(), "manga".to_string()],
                thumbnail: None,
            },
        ],
        tags: vec![
            Tag {
                slug: "illustration".to_string(),
                name: "イラスト".to_string(),
            },
            Tag {
                slug: "manga".to_string(),
                name: "漫画".to_string(),
            },
        ],
        series: vec![],
    };

    write_tag_pages_from_site_data(dir.path(), &data).unwrap();

    let illustration = fs::read_to_string(dir.path().join("tags/illustration/index.html")).unwrap();
    assert!(illustration.contains("作品1"));
    assert!(illustration.contains("作品2"));

    let manga = fs::read_to_string(dir.path().join("tags/manga/index.html")).unwrap();
    assert!(!manga.contains("作品1"));
    assert!(manga.contains("作品2"));
}
