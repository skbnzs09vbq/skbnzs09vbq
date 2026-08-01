//! 作品詳細静的ページ (`dist/works/<slug>/index.html`) 生成の統合テスト。
//!
//! 複数 Work を渡した際に、それぞれの `slug` に対応するディレクトリ・ファイルが
//! 正しく書き出され、内容にも各 Work の情報が反映されることを、crate 公開 API
//! (`topcoat::build::write_work_detail_pages`) を通して検証する。

use std::fs;

use topcoat::build::write_work_detail_pages;
use topcoat::models::{RelatedWorkRef, Series, Tag, Work};

fn work(slug: &str, title: &str) -> Work {
    Work {
        slug: slug.to_string(),
        title: title.to_string(),
        description: format!("{title} の説明"),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: None,
        tags: vec![Tag {
            slug: "shader".to_string(),
            name: "シェーダー".to_string(),
        }],
        series: Some(Series {
            slug: "series-1".to_string(),
            name: "シリーズ1".to_string(),
        }),
        params: serde_json::json!({ "seed": 1 }),
        related_works: vec![RelatedWorkRef {
            slug: "other-work".to_string(),
            title: "別の作品".to_string(),
        }],
    }
}

#[test]
fn writes_index_html_for_each_work_under_its_slug_directory() {
    let dist_dir = tempfile::tempdir().expect("should create tempdir");
    let tera = topcoat_render::build_tera().expect("template should load");
    let works = vec![work("work-1", "作品1"), work("work-2", "作品2")];

    write_work_detail_pages(dist_dir.path(), &tera, &works).expect("write should succeed");

    for slug in ["work-1", "work-2"] {
        let path = dist_dir.path().join("works").join(slug).join("index.html");
        assert!(path.exists(), "{path:?} should exist");
    }
}

#[test]
fn written_html_contains_work_specific_title_and_description() {
    let dist_dir = tempfile::tempdir().expect("should create tempdir");
    let tera = topcoat_render::build_tera().expect("template should load");
    let works = vec![work("work-1", "作品1"), work("work-2", "作品2")];

    write_work_detail_pages(dist_dir.path(), &tera, &works).expect("write should succeed");

    let html_1 = fs::read_to_string(dist_dir.path().join("works/work-1/index.html"))
        .expect("should read work-1 index.html");
    assert!(html_1.contains("作品1"));
    assert!(html_1.contains("作品1 の説明"));
    assert!(!html_1.contains("作品2"));

    let html_2 = fs::read_to_string(dist_dir.path().join("works/work-2/index.html"))
        .expect("should read work-2 index.html");
    assert!(html_2.contains("作品2"));
    assert!(html_2.contains("作品2 の説明"));
    assert!(!html_2.contains("作品1"));
}

#[test]
fn written_html_contains_tags_series_related_works_and_canvas_placeholder() {
    let dist_dir = tempfile::tempdir().expect("should create tempdir");
    let tera = topcoat_render::build_tera().expect("template should load");
    let works = vec![work("work-1", "作品1")];

    write_work_detail_pages(dist_dir.path(), &tera, &works).expect("write should succeed");

    let html = fs::read_to_string(dist_dir.path().join("works/work-1/index.html"))
        .expect("should read work-1 index.html");
    assert!(html.contains("/tags/shader/"));
    assert!(html.contains("/series/series-1/"));
    assert!(html.contains("/works/other-work/"));
    assert!(html.contains(r#"id="work-canvas""#));
    assert!(html.contains(r#"data-slug="work-1""#));
    assert!(html.contains("/works/work-1/og.png"));
}

#[test]
fn writes_nothing_when_no_works_are_given() {
    let dist_dir = tempfile::tempdir().expect("should create tempdir");
    let tera = topcoat_render::build_tera().expect("template should load");

    write_work_detail_pages(dist_dir.path(), &tera, &[]).expect("write should succeed");

    assert!(!dist_dir.path().join("works").exists());
}
