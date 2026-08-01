//! RSS 2.0 (`feed.xml`) / JSON Feed 1.1 (`feed.json`) 生成。
//!
//! 外部の XML/フィード生成 crate には依存しない:
//! - RSS は [`crate::xml_writer`] を使った手書き XML シリアライズ
//! - JSON Feed は `serde` の `Serialize` 実装 + `serde_json::to_string` で出力する
//!
//! いずれの関数も純粋関数であり、DB アクセスは行わない。呼び出し側 (`build::mod`) が
//! 新着順 (`created_at` 降順) に並べた [`FeedWork`] のスライスを渡す想定。

use serde::Serialize;

use crate::rfc822::rfc3339_to_rfc822;
use crate::xml_writer::{write_text_element, write_text_element_with_attr};

/// フィードに掲載する Work 1件分の情報。
///
/// [`crate::models::Work`] からのマッピングは呼び出し側 (`build::mod`) が行う。
#[derive(Debug, Clone)]
pub struct FeedWork {
    pub slug: String,
    pub title: String,
    pub description: String,
    /// RFC3339 (UTC) 形式
    pub created_at: String,
    /// RFC3339 (UTC) 形式。`None` の場合は `created_at` にフォールバックする。
    pub updated_at: Option<String>,
}

impl FeedWork {
    /// `updated_at` があればそれを、なければ `created_at` を返す。
    fn effective_updated_at(&self) -> &str {
        self.updated_at.as_deref().unwrap_or(&self.created_at)
    }
}

/// サイト全体の情報 (フィードの `<channel>` / トップレベルに載る情報)。
#[derive(Debug, Clone)]
pub struct FeedMeta {
    pub site_title: String,
    pub site_description: String,
    /// 末尾スラッシュなしのベース URL (例: `https://example.com`)
    pub base_url: String,
}

impl FeedMeta {
    fn work_link(&self, slug: &str) -> String {
        format!("{}/works/{}/", self.base_url, slug)
    }
}

/// `works` (新着順 = `created_at` 降順で並んでいる前提) から RSS 2.0 の XML 文字列を生成する。
///
/// この関数自体は並び替えを行わない。呼び出し側が新着順に並べたスライスを渡すこと。
pub fn generate_rss(meta: &FeedMeta, works: &[FeedWork]) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<rss version=\"2.0\">\n<channel>\n");

    write_text_element(&mut out, "title", &meta.site_title);
    write_text_element(&mut out, "link", &meta.base_url);
    write_text_element(&mut out, "description", &meta.site_description);

    if let Some(first) = works.first() {
        if let Some(rfc822) = rfc3339_to_rfc822(first.effective_updated_at()) {
            write_text_element(&mut out, "lastBuildDate", &rfc822);
        }
    }

    for work in works {
        out.push_str("<item>\n");
        write_text_element(&mut out, "title", &work.title);
        let link = meta.work_link(&work.slug);
        write_text_element(&mut out, "link", &link);
        write_text_element_with_attr(&mut out, "guid", "isPermaLink", "true", &link);
        if let Some(rfc822) = rfc3339_to_rfc822(work.effective_updated_at()) {
            write_text_element(&mut out, "pubDate", &rfc822);
        }
        write_text_element(&mut out, "description", &work.description);
        out.push_str("</item>\n");
    }

    out.push_str("</channel>\n</rss>\n");
    out
}

#[derive(Debug, Serialize)]
struct JsonFeedDocument {
    version: &'static str,
    title: String,
    home_page_url: String,
    feed_url: String,
    description: String,
    items: Vec<JsonFeedItem>,
}

#[derive(Debug, Serialize)]
struct JsonFeedItem {
    id: String,
    url: String,
    title: String,
    content_text: String,
    date_published: String,
    date_modified: String,
}

/// `works` (新着順 = `created_at` 降順で並んでいる前提) から JSON Feed 1.1 の JSON 文字列を生成する。
pub fn generate_json_feed(meta: &FeedMeta, works: &[FeedWork]) -> String {
    let items = works
        .iter()
        .map(|work| {
            let link = meta.work_link(&work.slug);
            JsonFeedItem {
                id: link.clone(),
                url: link,
                title: work.title.clone(),
                content_text: work.description.clone(),
                date_published: work.created_at.clone(),
                date_modified: work.effective_updated_at().to_string(),
            }
        })
        .collect();

    let document = JsonFeedDocument {
        version: "https://jsonfeed.org/version/1.1",
        title: meta.site_title.clone(),
        home_page_url: meta.base_url.clone(),
        feed_url: format!("{}/feed.json", meta.base_url),
        description: meta.site_description.clone(),
        items,
    };

    // JsonFeedDocument は素朴なフィールド (String / &'static str / Vec) のみで構成されており、
    // シリアライズが失敗するケースは存在しないため unwrap_or_default で十分。
    serde_json::to_string_pretty(&document).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> FeedMeta {
        FeedMeta {
            site_title: "Test Site".to_string(),
            site_description: "desc".to_string(),
            base_url: "https://example.com".to_string(),
        }
    }

    #[test]
    fn rss_empty_works_has_no_items() {
        let xml = generate_rss(&meta(), &[]);
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<rss version=\"2.0\">"));
        assert!(xml.contains("<channel>"));
        assert!(!xml.contains("<item>"));
    }

    #[test]
    fn rss_single_work_contains_expected_fields() {
        let works = vec![FeedWork {
            slug: "work-1".into(),
            title: "作品1".into(),
            description: "説明".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: None,
        }];
        let xml = generate_rss(&meta(), &works);
        assert!(xml.contains("<link>https://example.com/works/work-1/</link>\n"));
        assert!(
            xml.contains("<guid isPermaLink=\"true\">https://example.com/works/work-1/</guid>\n")
        );
        assert!(xml.contains("<pubDate>Thu, 01 Jan 2026 00:00:00 GMT</pubDate>\n"));
        assert!(xml.contains("<description>説明</description>\n"));
    }

    #[test]
    fn rss_preserves_caller_supplied_order() {
        let works = vec![
            FeedWork {
                slug: "a".into(),
                title: "A".into(),
                description: "".into(),
                created_at: "2026-01-02T00:00:00Z".into(),
                updated_at: None,
            },
            FeedWork {
                slug: "b".into(),
                title: "B".into(),
                description: "".into(),
                created_at: "2026-01-01T00:00:00Z".into(),
                updated_at: None,
            },
        ];
        let xml = generate_rss(&meta(), &works);
        let pos_a = xml.find("works/a/").unwrap();
        let pos_b = xml.find("works/b/").unwrap();
        assert!(pos_a < pos_b);
    }

    #[test]
    fn rss_escapes_special_characters_in_title() {
        let works = vec![FeedWork {
            slug: "special".into(),
            title: "<Title> & \"quote\"".into(),
            description: "desc".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: None,
        }];
        let xml = generate_rss(&meta(), &works);
        assert!(xml.contains("&lt;Title&gt; &amp; \"quote\""));
        assert!(!xml.contains("<Title>"));
    }

    #[test]
    fn rss_updated_at_overrides_pub_date() {
        let works = vec![FeedWork {
            slug: "work-1".into(),
            title: "Work".into(),
            description: "".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: Some("2026-02-15T12:30:00Z".into()),
        }];
        let xml = generate_rss(&meta(), &works);
        assert!(xml.contains("<pubDate>Sun, 15 Feb 2026 12:30:00 GMT</pubDate>\n"));
    }

    #[test]
    fn json_feed_empty_works_has_empty_items() {
        let json = generate_json_feed(&meta(), &[]);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["version"], "https://jsonfeed.org/version/1.1");
        assert_eq!(value["items"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn json_feed_single_work_fields() {
        let works = vec![FeedWork {
            slug: "work-1".into(),
            title: "Title & <Tag>".into(),
            description: "desc".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: Some("2026-01-02T00:00:00Z".into()),
        }];
        let json = generate_json_feed(&meta(), &works);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let item = &value["items"][0];
        assert_eq!(item["id"], "https://example.com/works/work-1/");
        assert_eq!(item["url"], "https://example.com/works/work-1/");
        // JSON では XML エスケープ不要 (serde_json が JSON 用エスケープを行う)
        assert_eq!(item["title"], "Title & <Tag>");
        assert_eq!(item["date_published"], "2026-01-01T00:00:00Z");
        assert_eq!(item["date_modified"], "2026-01-02T00:00:00Z");
    }

    #[test]
    fn json_feed_falls_back_to_created_at_when_updated_at_missing() {
        let works = vec![FeedWork {
            slug: "work-1".into(),
            title: "Title".into(),
            description: "".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: None,
        }];
        let json = generate_json_feed(&meta(), &works);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["items"][0]["date_modified"], "2026-01-01T00:00:00Z");
    }
}
