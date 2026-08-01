//! feed.xml (RSS 2.0) / feed.json (JSON Feed 1.1) 生成の統合テスト。
//!
//! 0件・1件・複数件・特殊文字を含む title 等のケースを、crate 公開 API
//! (`topcoat::build::feed`) を通して検証する。

use topcoat::build::feed::{generate_json_feed, generate_rss, FeedMeta, FeedWork};

fn meta() -> FeedMeta {
    FeedMeta {
        site_title: "サンプルサイト".to_string(),
        site_description: "サイトの説明".to_string(),
        base_url: "https://example.com".to_string(),
    }
}

fn work(slug: &str, title: &str, created_at: &str, updated_at: Option<&str>) -> FeedWork {
    FeedWork {
        slug: slug.to_string(),
        title: title.to_string(),
        description: format!("{title} の説明"),
        created_at: created_at.to_string(),
        updated_at: updated_at.map(str::to_string),
    }
}

#[test]
fn rss_zero_works_is_well_formed_with_no_items() {
    let xml = generate_rss(&meta(), &[]);
    assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(xml.contains("<rss version=\"2.0\">"));
    assert!(xml.contains("<channel>"));
    assert!(xml.contains("</channel>"));
    assert!(xml.contains("</rss>"));
    assert_eq!(xml.matches("<item>").count(), 0);
    // channel 内の必須フィールド
    assert!(xml.contains("<title>サンプルサイト</title>"));
    assert!(xml.contains("<description>サイトの説明</description>"));
}

#[test]
fn rss_one_work_includes_single_item_with_required_fields() {
    let works = vec![work("work-1", "作品タイトル", "2026-03-10T09:00:00Z", None)];
    let xml = generate_rss(&meta(), &works);
    assert_eq!(xml.matches("<item>").count(), 1);
    assert!(xml.contains("<title>作品タイトル</title>"));
    assert!(xml.contains("<link>https://example.com/works/work-1/</link>"));
    assert!(xml.contains("isPermaLink=\"true\">https://example.com/works/work-1/</guid>"));
    assert!(xml.contains("<pubDate>"));
    assert!(xml.contains("<description>作品タイトル の説明</description>"));
}

#[test]
fn rss_multiple_works_include_all_items_in_input_order() {
    let works = vec![
        work("work-3", "3番目に新しい", "2026-03-01T00:00:00Z", None),
        work("work-2", "2番目に新しい", "2026-02-01T00:00:00Z", None),
        work("work-1", "1番目に新しい", "2026-01-01T00:00:00Z", None),
    ];
    let xml = generate_rss(&meta(), &works);
    assert_eq!(xml.matches("<item>").count(), 3);

    let pos_3 = xml.find("works/work-3/").unwrap();
    let pos_2 = xml.find("works/work-2/").unwrap();
    let pos_1 = xml.find("works/work-1/").unwrap();
    assert!(
        pos_3 < pos_2 && pos_2 < pos_1,
        "新着順 (呼び出し側が渡した順) が保持されること"
    );
}

#[test]
fn rss_escapes_special_characters_in_title_and_description() {
    let works = vec![FeedWork {
        slug: "special".to_string(),
        title: "<script> & \"quote\" 'apos'".to_string(),
        description: "A < B & C > D".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: None,
    }];
    let xml = generate_rss(&meta(), &works);
    assert!(!xml.contains("<script>"));
    assert!(xml.contains("&lt;script&gt; &amp; \"quote\" 'apos'"));
    assert!(xml.contains("A &lt; B &amp; C &gt; D"));
}

#[test]
fn json_feed_zero_works_has_empty_items_array_and_valid_shape() {
    let json = generate_json_feed(&meta(), &[]);
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(value["version"], "https://jsonfeed.org/version/1.1");
    assert_eq!(value["title"], "サンプルサイト");
    assert_eq!(value["home_page_url"], "https://example.com");
    assert_eq!(value["feed_url"], "https://example.com/feed.json");
    assert!(value["items"].as_array().unwrap().is_empty());
}

#[test]
fn json_feed_one_work_has_expected_item_fields() {
    let works = vec![work(
        "work-1",
        "作品タイトル",
        "2026-03-10T09:00:00Z",
        Some("2026-03-11T10:00:00Z"),
    )];
    let json = generate_json_feed(&meta(), &works);
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    let items = value["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item["id"], "https://example.com/works/work-1/");
    assert_eq!(item["url"], "https://example.com/works/work-1/");
    assert_eq!(item["title"], "作品タイトル");
    assert_eq!(item["date_published"], "2026-03-10T09:00:00Z");
    assert_eq!(item["date_modified"], "2026-03-11T10:00:00Z");
}

#[test]
fn json_feed_multiple_works_preserve_input_order_and_count() {
    let works = vec![
        work("work-3", "3", "2026-03-01T00:00:00Z", None),
        work("work-2", "2", "2026-02-01T00:00:00Z", None),
        work("work-1", "1", "2026-01-01T00:00:00Z", None),
    ];
    let json = generate_json_feed(&meta(), &works);
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    let items = value["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0]["id"], "https://example.com/works/work-3/");
    assert_eq!(items[1]["id"], "https://example.com/works/work-2/");
    assert_eq!(items[2]["id"], "https://example.com/works/work-1/");
}

#[test]
fn json_feed_does_not_double_escape_special_characters() {
    let works = vec![FeedWork {
        slug: "special".to_string(),
        title: "<script> & \"quote\"".to_string(),
        description: "desc".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: None,
    }];
    let json = generate_json_feed(&meta(), &works);
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    // JSON の値としては元の文字列がそのまま入っている (JSON エンコード上のエスケープのみ)
    assert_eq!(value["items"][0]["title"], "<script> & \"quote\"");
}

#[test]
fn json_feed_falls_back_to_created_at_when_updated_at_is_absent() {
    let works = vec![work("work-1", "タイトル", "2026-01-01T00:00:00Z", None)];
    let json = generate_json_feed(&meta(), &works);
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(value["items"][0]["date_modified"], "2026-01-01T00:00:00Z");
}
