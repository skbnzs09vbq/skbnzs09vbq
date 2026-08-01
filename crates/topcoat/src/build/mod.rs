//! `topcoat build` のビルドパイプライン。
//!
//! マイグレーション適用 → topcoat-db 経由の Work/Tag/Series/Version 取得 → Tera による
//! レンダリング → feed/sitemap 生成 → OG 画像生成 → `dist/` 配下への書き出し、までの
//! 一連の処理を [`run`] に配線している。
//!
//! Work/Tag/Series/Version の実テーブル定義は #6 (Work テーブル) / #9 (タグ別一覧) /
//! #10 (シリーズ別一覧) が未マージのため、[`fetch_site_data`] は現時点では空データを
//! 返すプレースホルダになっている。上記 issue のマージ後は、Diesel 経由の実データ取得
//! 処理に差し替える。

pub mod feed;
pub mod sitemap;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::models::{Series, Tag, Version, Work};
use feed::{generate_json_feed, generate_rss, FeedMeta, FeedWork};
use sitemap::{
    generate_sitemap, SitemapInput, SitemapSeriesEntry, SitemapTagEntry, SitemapWorkEntry,
};
use topcoat_render::OgWork;

/// `topcoat build` が feed/sitemap/ページ生成に必要とする全データ。
///
/// `works` は新着順 (`created_at` 降順) に並んでいる前提。
#[derive(Debug, Clone, Default)]
pub struct SiteData {
    pub works: Vec<Work>,
    pub tags: Vec<Tag>,
    pub series: Vec<Series>,
    pub versions: Vec<Version>,
}

/// サイト全体で共有される設定値。
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// 末尾スラッシュなしのベース URL (例: `https://example.com`)
    pub base_url: String,
    pub site_title: String,
    pub site_description: String,
    /// マイグレーション適用先の SQLite ファイルパス。
    ///
    /// 本番では `topcoat_db::database_path()`（固定パス）を、テストでは
    /// `tempfile` 等で払い出した一時パスを渡す。
    pub db_path: PathBuf,
}

/// マイグレーション適用 → (プレースホルダの) Work/Tag/Series/Version 取得 → レンダリング →
/// feed/sitemap 生成 → OG 画像生成 → `dist_dir` 配下への書き出し、までの一連のビルド
/// パイプラインを実行する。
pub fn run(dist_dir: &Path, config: &BuildConfig) -> io::Result<()> {
    apply_migrations(&config.db_path).map_err(io::Error::other)?;

    let data = fetch_site_data();

    fs::create_dir_all(dist_dir)?;

    let index_html = topcoat_render::render_index().map_err(io::Error::other)?;
    fs::write(dist_dir.join("index.html"), index_html)?;

    write_feeds_and_sitemap(
        dist_dir,
        &config.base_url,
        &config.site_title,
        &config.site_description,
        &data,
    )?;

    write_og_images(dist_dir, &data)
}

/// `db_path` の SQLite ファイルへの接続を確立し、埋め込みマイグレーションを適用する。
fn apply_migrations(
    db_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut connection = topcoat_db::establish_connection_at(db_path)?;
    topcoat_db::run_migrations(&mut connection)
}

/// topcoat-db 経由で Work/Tag/Series/Version を全件取得する。
///
/// TODO(#6, #9, #10): 実テーブル定義のマージ後、Diesel 経由の実データ取得に差し替える。
/// 現時点ではテーブル未定義のため、空データを返すプレースホルダとする。
fn fetch_site_data() -> SiteData {
    SiteData::default()
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

/// `dist_dir/works/<slug>/og.png` を各 Work ごとに書き出す。
///
/// `topcoat_render::OgWork` へのマッピングは呼び出し側 (この関数) が行う
/// ([`FeedWork`] と同じ「Work → 専用 DTO へのマッピングは呼び出し側が行う」パターン)。
pub fn write_og_images(dist_dir: &Path, data: &SiteData) -> io::Result<()> {
    let og_works: Vec<OgWork> = data
        .works
        .iter()
        .map(|work| OgWork {
            slug: work.slug.clone(),
            title: work.title.clone(),
        })
        .collect();

    topcoat_render::write_og_images(dist_dir, &og_works)?;

    Ok(())
}
