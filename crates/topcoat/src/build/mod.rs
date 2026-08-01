//! `topcoat build` のビルドパイプライン。
//!
//! 本来このモジュールの骨格は #11 (SSG ビルドエントリーポイント実装) で作成され、
//! 全 Work/Tag/Series 取得フックは #4 (SQLite + Diesel 導入) / #6 (Work テーブル) /
//! #9 (タグ別一覧) / #10 (シリーズ別一覧) で実装される想定だが、issue #15 着手時点で
//! いずれも未マージのため、本 issue のスコープ内で完結させるために最小限の
//! [`SiteData`] 受け渡しインターフェースを暫定的に用意している。
//!
//! 後続 issue のマージ後にやること:
//! - [`SiteData`] を Diesel 経由の実データ取得結果から構築する処理に差し替える
//! - `main.rs` 側のプレースホルダ (`SiteData::default()` 相当) を実データ取得呼び出しに差し替える

pub mod feed;
pub mod sitemap;

use std::fs;
use std::io;
use std::path::Path;

use crate::models::{Series, Tag, Work};
use feed::{generate_json_feed, generate_rss, FeedMeta, FeedWork};
use sitemap::{
    generate_sitemap, SitemapInput, SitemapSeriesEntry, SitemapTagEntry, SitemapWorkEntry,
};

/// `topcoat build` が feed/sitemap 生成に必要とする全データ。
///
/// `works` は新着順 (`created_at` 降順) に並んでいる前提。
#[derive(Debug, Clone, Default)]
pub struct SiteData {
    pub works: Vec<Work>,
    pub tags: Vec<Tag>,
    pub series: Vec<Series>,
}

/// `dist/feed.xml` / `dist/feed.json` / `dist/sitemap.xml` を `dist_dir` 配下に書き出す。
///
/// `site_title` / `site_description` / `base_url` はサイト全体の設定値であり、
/// 呼び出し側 (`main.rs`) が一元的に管理する。
pub fn write_feeds_and_sitemap(
    dist_dir: &Path,
    base_url: &str,
    site_title: &str,
    site_description: &str,
    data: &SiteData,
) -> io::Result<()> {
    fs::create_dir_all(dist_dir)?;

    let meta = FeedMeta {
        site_title: site_title.to_string(),
        site_description: site_description.to_string(),
        base_url: base_url.trim_end_matches('/').to_string(),
    };

    let feed_works: Vec<FeedWork> = data
        .works
        .iter()
        .map(|work| FeedWork {
            slug: work.slug.clone(),
            title: work.title.clone(),
            description: work.description.clone(),
            created_at: work.created_at.clone(),
            updated_at: work.updated_at.clone(),
        })
        .collect();

    let rss_xml = generate_rss(&meta, &feed_works);
    let json_feed = generate_json_feed(&meta, &feed_works);

    let sitemap_input = SitemapInput {
        base_url: meta.base_url.clone(),
        works: data
            .works
            .iter()
            .map(|work| SitemapWorkEntry {
                slug: work.slug.clone(),
                lastmod: Some(work.effective_updated_at().to_string()),
            })
            .collect(),
        tags: data
            .tags
            .iter()
            .map(|tag| SitemapTagEntry {
                slug: tag.slug.clone(),
            })
            .collect(),
        series: data
            .series
            .iter()
            .map(|series| SitemapSeriesEntry {
                slug: series.slug.clone(),
            })
            .collect(),
    };
    let sitemap_xml = generate_sitemap(&sitemap_input);

    fs::write(dist_dir.join("feed.xml"), rss_xml)?;
    fs::write(dist_dir.join("feed.json"), json_feed)?;
    fs::write(dist_dir.join("sitemap.xml"), sitemap_xml)?;

    Ok(())
}
