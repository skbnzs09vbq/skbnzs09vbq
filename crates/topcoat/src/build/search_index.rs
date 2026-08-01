//! クライアント検索用インデックス (`search-index.json`) 生成。
//!
//! `serde` の `Serialize` 実装 + `serde_json::to_string_pretty` で出力する。
//!
//! この関数は純粋関数であり、DB アクセスは行わない。呼び出し側 (`build::mod`) が
//! [`crate::models::Work`] から変換した [`SearchIndexEntry`] のスライスを渡す想定。

use serde::Serialize;

/// 検索インデックスに載せる Work 1件分の情報。
///
/// [`crate::models::Work`] からのマッピングは呼び出し側 (`build::mod`) が行う。
#[derive(Debug, Clone, Serialize)]
pub struct SearchIndexEntry {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub series: Option<String>,
}

/// `entries` から `search-index.json` の JSON 文字列を生成する。
///
/// この関数自体は並び替えを行わない。呼び出し側が渡した順序をそのまま保持する。
pub fn generate_search_index(entries: &[SearchIndexEntry]) -> String {
    // SearchIndexEntry は素朴なフィールド (String / Vec<String> / Option<String>) のみで
    // 構成されており、シリアライズが失敗するケースは存在しないため unwrap_or_default で十分。
    serde_json::to_string_pretty(entries).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn zero_entries_produces_empty_array() {
        let json = generate_search_index(&[]);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value.as_array().unwrap().len(), 0);
    }

    #[test]
    fn single_entry_contains_expected_fields() {
        let entries = vec![entry(
            "work-1",
            "作品1",
            vec!["イラスト", "オリジナル"],
            Some("シリーズA"),
        )];
        let json = generate_search_index(&entries);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let item = &value[0];
        assert_eq!(item["slug"], "work-1");
        assert_eq!(item["title"], "作品1");
        assert_eq!(item["description"], "作品1 の説明");
        assert_eq!(item["tags"], serde_json::json!(["イラスト", "オリジナル"]));
        assert_eq!(item["series"], "シリーズA");
    }

    #[test]
    fn multiple_entries_preserve_input_order_and_count() {
        let entries = vec![
            entry("work-1", "1", vec![], None),
            entry("work-2", "2", vec![], None),
            entry("work-3", "3", vec![], None),
        ];
        let json = generate_search_index(&entries);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let array = value.as_array().unwrap();
        assert_eq!(array.len(), 3);
        assert_eq!(array[0]["slug"], "work-1");
        assert_eq!(array[1]["slug"], "work-2");
        assert_eq!(array[2]["slug"], "work-3");
    }

    #[test]
    fn empty_tags_serializes_to_empty_array() {
        let entries = vec![entry("work-1", "タイトル", vec![], Some("シリーズA"))];
        let json = generate_search_index(&entries);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value[0]["tags"], serde_json::json!([]));
    }

    #[test]
    fn none_series_serializes_to_null() {
        let entries = vec![entry("work-1", "タイトル", vec!["タグ"], None)];
        let json = generate_search_index(&entries);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value[0]["series"].is_null());
    }
}
