//! シリーズ別一覧ページ (`dist/series/<slug>/index.html`) 生成の統合テスト。
//!
//! 0件・複数件（作成順ソート）・他シリーズ混入なし・特殊文字エスケープ・
//! 複数 Series での複数ページ生成を、crate 公開 API (`topcoat::build::series`) を
//! 通して検証する。

use topcoat::build::series::{
    render_series_page, write_series_pages, SeriesPageInput, SeriesPageWork,
};
use topcoat::models::{Series, Work};

fn work(slug: &str, series_slug: &str, created_at: &str) -> Work {
    Work {
        slug: slug.to_string(),
        title: format!("Title {slug}"),
        description: format!("Description {slug}"),
        created_at: created_at.to_string(),
        updated_at: None,
        tags: vec![],
        series: None,
        series_slug: Some(series_slug.to_string()),
        params: serde_json::Value::Null,
        related_works: vec![],
    }
}

#[test]
fn series_with_zero_works_renders_without_crashing() {
    let dist_dir = tempfile::tempdir().expect("一時ディレクトリの作成に失敗しました");
    let series = vec![Series {
        slug: "no-works".to_string(),
        name: "作品なしシリーズ".to_string(),
    }];

    write_series_pages(dist_dir.path(), &[], &series).expect("空一覧の書き出しに失敗しました");

    let html = std::fs::read_to_string(dist_dir.path().join("series/no-works/index.html")).unwrap();
    assert!(html.contains("作品なしシリーズ"));
}

#[test]
fn multiple_works_are_listed_in_ascending_created_at_order_regardless_of_input_order() {
    let dist_dir = tempfile::tempdir().expect("一時ディレクトリの作成に失敗しました");
    let series = vec![Series {
        slug: "series-1".to_string(),
        name: "Series 1".to_string(),
    }];
    // わざと降順で渡す
    let works = vec![
        work("work-c", "series-1", "2026-03-01T00:00:00Z"),
        work("work-a", "series-1", "2026-01-01T00:00:00Z"),
        work("work-b", "series-1", "2026-02-01T00:00:00Z"),
    ];

    write_series_pages(dist_dir.path(), &works, &series).unwrap();

    let html = std::fs::read_to_string(dist_dir.path().join("series/series-1/index.html")).unwrap();
    let pos_a = html.find("work-a").expect("work-a should be present");
    let pos_b = html.find("work-b").expect("work-b should be present");
    let pos_c = html.find("work-c").expect("work-c should be present");
    assert!(pos_a < pos_b, "work-a should come before work-b");
    assert!(pos_b < pos_c, "work-b should come before work-c");
}

#[test]
fn works_belonging_to_other_series_are_not_included() {
    let dist_dir = tempfile::tempdir().expect("一時ディレクトリの作成に失敗しました");
    let series = vec![
        Series {
            slug: "series-a".to_string(),
            name: "Series A".to_string(),
        },
        Series {
            slug: "series-b".to_string(),
            name: "Series B".to_string(),
        },
    ];
    let works = vec![
        work("a-work", "series-a", "2026-01-01T00:00:00Z"),
        work("b-work", "series-b", "2026-01-01T00:00:00Z"),
    ];

    write_series_pages(dist_dir.path(), &works, &series).unwrap();

    let html_a =
        std::fs::read_to_string(dist_dir.path().join("series/series-a/index.html")).unwrap();
    assert!(html_a.contains("a-work"));
    assert!(!html_a.contains("b-work"));

    let html_b =
        std::fs::read_to_string(dist_dir.path().join("series/series-b/index.html")).unwrap();
    assert!(html_b.contains("b-work"));
    assert!(!html_b.contains("a-work"));
}

#[test]
fn title_and_description_special_characters_are_html_escaped() {
    let input = SeriesPageInput {
        series_name: "Series 1".to_string(),
        works: vec![SeriesPageWork {
            slug: "work-1".to_string(),
            title: "<script>alert(1)</script> & \"quote\"".to_string(),
            description: "A & B <tag>".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
        }],
    };

    let html = render_series_page(&input);

    // Tera の autoescape は `<`/`>`/`&`/`"`/`'`/`/` をエスケープする。
    assert!(html.contains("&lt;script&gt;alert(1)&lt;&#x2F;script&gt;"));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("A &amp; B &lt;tag&gt;"));
    assert!(!html.contains("A & B <tag>"));
}

#[test]
fn each_series_gets_its_own_index_html_at_dist_series_slug() {
    let dist_dir = tempfile::tempdir().expect("一時ディレクトリの作成に失敗しました");
    let series = vec![
        Series {
            slug: "slug-a".to_string(),
            name: "Series A".to_string(),
        },
        Series {
            slug: "slug-b".to_string(),
            name: "Series B".to_string(),
        },
    ];

    write_series_pages(dist_dir.path(), &[], &series).unwrap();

    assert!(dist_dir.path().join("series/slug-a/index.html").is_file());
    assert!(dist_dir.path().join("series/slug-b/index.html").is_file());
}
