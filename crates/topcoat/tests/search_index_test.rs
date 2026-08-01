//! search-index.json (クライアント検索用インデックス) 生成の統合テスト。
//!
//! 0件・1件・複数件・`tags` が空配列・`series` が `None` のケースを、
//! crate 公開 API (`topcoat::build::search_index`) を通して検証する。

use topcoat::build::search_index::{generate_search_index, SearchIndexEntry};

fn entry(slug: &str, title: &str, tags: Vec<&str>, series: Option<&str>) -> SearchIndexEntry {
    SearchIndexEntry {
        slug: slug.to_string(),
        title: title.to_string(),
        description: format!("{title} の説明"),
        tags: tags.into_iter().map(str::to_string).collect(),
        series: series.map(str::to_string),
    }
}

#[test]
fn zero_entries_produces_empty_json_array() {
    let json = generate_search_index(&[]);
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(value.is_array());
    assert_eq!(value.as_array().unwrap().len(), 0);
}

#[test]
fn one_entry_has_expected_fields() {
    let entries = vec![entry(
        "work-1",
        "作品タイトル",
        vec!["イラスト", "オリジナル"],
        Some("シリーズA"),
    )];
    let json = generate_search_index(&entries);
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    let items = value.as_array().unwrap();
    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item["slug"], "work-1");
    assert_eq!(item["title"], "作品タイトル");
    assert_eq!(item["description"], "作品タイトル の説明");
    assert_eq!(item["tags"], serde_json::json!(["イラスト", "オリジナル"]));
    assert_eq!(item["series"], "シリーズA");
}

#[test]
fn multiple_entries_preserve_input_order_and_count() {
    let entries = vec![
        entry("work-3", "3番目", vec!["タグA"], None),
        entry("work-2", "2番目", vec!["タグB"], Some("シリーズB")),
        entry("work-1", "1番目", vec![], None),
    ];
    let json = generate_search_index(&entries);
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    let items = value.as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0]["slug"], "work-3");
    assert_eq!(items[1]["slug"], "work-2");
    assert_eq!(items[2]["slug"], "work-1");
}

#[test]
fn empty_tags_serializes_to_empty_json_array() {
    let entries = vec![entry("work-1", "タイトル", vec![], Some("シリーズA"))];
    let json = generate_search_index(&entries);
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(value[0]["tags"].as_array().unwrap().is_empty());
}

#[test]
fn none_series_serializes_to_json_null() {
    let entries = vec![entry("work-1", "タイトル", vec!["タグ"], None)];
    let json = generate_search_index(&entries);
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(value[0]["series"].is_null());
}

#[test]
fn escapes_are_handled_by_json_encoding_not_double_escaped() {
    let entries = vec![entry(
        "special",
        "<script> & \"quote\"",
        vec!["<tag>"],
        None,
    )];
    let json = generate_search_index(&entries);
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(value[0]["title"], "<script> & \"quote\"");
    assert_eq!(value[0]["tags"][0], "<tag>");
}
