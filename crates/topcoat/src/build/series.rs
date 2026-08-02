//! シリーズ別一覧ページ (`dist/series/<slug>/index.html`) 生成。
//!
//! 各 [`crate::models::Series`] について、`series_slug` が一致する
//! [`crate::models::Work`] を作成順 (`created_at` 昇順) に列挙した静的 HTML を
//! [`tera`] でレンダリングする。
//!
//! テンプレートは `templates/series.html.tera` を `include_str!` でバイナリに
//! 埋め込み、`Tera::default()` + `add_raw_template` + `render`（`Tera::one_off`
//! 相当の挙動）でレンダリングする。ディレクトリ glob 方式
//! (`Tera::new("templates/**/*")`) を採らないのは、ビルド成果物である `topcoat`
//! バイナリの実行時カレントディレクトリに依存させず、テンプレートファイルの
//! 配置漏れによる実行時エラーを避けるため。テンプレートのパースは初回呼び出し時
//! 一度だけ行い、以降は [`std::sync::OnceLock`] にキャッシュした `Tera` を使い回す。
//!
//! `topcoat-render` crate はテンプレートレンダリングを担う想定の crate だが、
//! 現時点では stub のみで他 issue (#13 OG画像生成等) との統合方針が未確定のため、
//! `feed.rs`/`sitemap.rs` に倣い本 crate 内で自己完結させている。将来的に
//! テンプレートを集約するタイミングで `topcoat-render` へ移設する可能性がある。
//!
//! [`write_series_pages`] 以外は純粋関数であり、DB アクセスは行わない。

use std::fs;
use std::io;
use std::path::Path;
use std::sync::OnceLock;

use serde::Serialize;
use tera::{Context, Tera};

use crate::models::{Series, Work};

const SERIES_TEMPLATE: &str = include_str!("templates/series.html.tera");
const SERIES_TEMPLATE_NAME: &str = "series.html.tera";

/// [`SERIES_TEMPLATE`] を登録済みの `Tera` インスタンス。
///
/// series の件数分 [`render_series_page`] が呼ばれても、テンプレートのパースは
/// プロセス内で一度だけ行われる。
static SERIES_TERA: OnceLock<Tera> = OnceLock::new();

fn series_tera() -> &'static Tera {
    SERIES_TERA.get_or_init(|| {
        let mut tera = Tera::default();
        // autoescape はデフォルトで有効だが、`Tera::default()` はどの拡張子にも
        // マッチしないため、`one_off` 相当の挙動として明示的に登録して有効化する。
        tera.autoescape_on(vec![".html.tera"]);
        tera.add_raw_template(SERIES_TEMPLATE_NAME, SERIES_TEMPLATE)
            .expect(
                "series.html.tera はビルド時に埋め込まれる固定テンプレートのため常にパースに成功する",
            );
        tera
    })
}

/// シリーズ一覧ページに掲載する Work 1件分の情報。
///
/// [`crate::models::Work`] からのマッピングは呼び出し側 ([`write_series_pages`]) が行う。
/// `Serialize` は Tera の [`Context::insert`] に渡すために必要。
#[derive(Debug, Clone, Serialize)]
pub struct SeriesPageWork {
    pub slug: String,
    pub title: String,
    pub description: String,
    /// RFC3339 (UTC) 形式
    pub created_at: String,
}

/// シリーズ一覧ページのレンダリングへの入力。
///
/// `works` は作成順 (`created_at` 昇順) に並んでいる前提。この struct 自体は
/// 並び替えを行わない。
#[derive(Debug, Clone)]
pub struct SeriesPageInput {
    pub series_slug: String,
    pub series_name: String,
    pub works: Vec<SeriesPageWork>,
}

/// `input` からシリーズ一覧ページの HTML 文字列を生成する。
///
/// この関数自体は並び替え・フィルタリングを行わない。呼び出し側が
/// 対象シリーズに属する Work を作成順に並べたスライスを渡すこと。
pub fn render_series_page(input: &SeriesPageInput) -> String {
    let mut context = Context::new();
    context.insert("series_name", &input.series_name);
    context.insert("works", &input.works);

    series_tera()
        .render(SERIES_TEMPLATE_NAME, &context)
        .expect("context のキーはテンプレートと一致しており、レンダリングは常に成功する")
}

/// `works` から `series` それぞれに属するものを作成順 (`created_at` 昇順) に集め、
/// `dist_dir/series/<slug>/index.html` へ書き出す。
///
/// 所属 Work が0件の Series でも空一覧として出力する。出力先ディレクトリは
/// Series ごとに都度 `fs::create_dir_all` で作成する。
pub fn write_series_pages(dist_dir: &Path, works: &[Work], series: &[Series]) -> io::Result<()> {
    for s in series {
        let mut series_works: Vec<&Work> = works
            .iter()
            .filter(|work| work.series_slug.as_deref() == Some(s.slug.as_str()))
            .collect();
        series_works.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        let input = SeriesPageInput {
            series_slug: s.slug.clone(),
            series_name: s.name.clone(),
            works: series_works
                .into_iter()
                .map(|work| SeriesPageWork {
                    slug: work.slug.clone(),
                    title: work.title.clone(),
                    description: work.description.clone(),
                    created_at: work.created_at.clone(),
                })
                .collect(),
        };

        let html = render_series_page(&input);

        let out_dir = dist_dir.join("series").join(&s.slug);
        fs::create_dir_all(&out_dir)?;
        fs::write(out_dir.join("index.html"), html)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn work(slug: &str, series_slug: &str, created_at: &str) -> Work {
        Work {
            slug: slug.to_string(),
            title: format!("Title {slug}"),
            description: format!("Description {slug}"),
            created_at: created_at.to_string(),
            updated_at: None,
            tags: vec![],
            thumbnail: None,
            series: None,
            series_slug: Some(series_slug.to_string()),
            params: serde_json::Value::Null,
            related_works: vec![],
        }
    }

    #[test]
    fn render_empty_works_produces_empty_list_message() {
        let input = SeriesPageInput {
            series_slug: "series-a".to_string(),
            series_name: "Series A".to_string(),
            works: vec![],
        };
        let html = render_series_page(&input);
        assert!(html.contains("Series A"));
        assert!(!html.contains("<li"));
    }

    #[test]
    fn render_escapes_special_characters() {
        let input = SeriesPageInput {
            series_slug: "series-a".to_string(),
            series_name: "Series A".to_string(),
            works: vec![SeriesPageWork {
                slug: "work-1".to_string(),
                title: "<Title> & \"quote\"".to_string(),
                description: "<desc>".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
            }],
        };
        let html = render_series_page(&input);
        assert!(html.contains("&lt;Title&gt;"));
        assert!(!html.contains("<Title>"));
        assert!(html.contains("&lt;desc&gt;"));
    }

    #[test]
    fn write_series_pages_creates_empty_list_for_series_with_no_works() {
        let dist_dir = tempfile::tempdir().expect("一時ディレクトリの作成に失敗しました");
        let series = vec![Series {
            slug: "empty-series".to_string(),
            name: "Empty Series".to_string(),
        }];

        write_series_pages(dist_dir.path(), &[], &series).unwrap();

        let html =
            fs::read_to_string(dist_dir.path().join("series/empty-series/index.html")).unwrap();
        assert!(html.contains("Empty Series"));
    }

    #[test]
    fn write_series_pages_orders_works_by_created_at_ascending_even_if_input_is_descending() {
        let dist_dir = tempfile::tempdir().expect("一時ディレクトリの作成に失敗しました");
        let series = vec![Series {
            slug: "s".to_string(),
            name: "S".to_string(),
        }];
        // 降順で渡す
        let works = vec![
            work("newest", "s", "2026-03-01T00:00:00Z"),
            work("oldest", "s", "2026-01-01T00:00:00Z"),
            work("middle", "s", "2026-02-01T00:00:00Z"),
        ];

        write_series_pages(dist_dir.path(), &works, &series).unwrap();

        let html = fs::read_to_string(dist_dir.path().join("series/s/index.html")).unwrap();
        let pos_oldest = html.find("oldest").expect("oldest work should be present");
        let pos_middle = html.find("middle").expect("middle work should be present");
        let pos_newest = html.find("newest").expect("newest work should be present");
        assert!(pos_oldest < pos_middle);
        assert!(pos_middle < pos_newest);
    }

    #[test]
    fn write_series_pages_does_not_leak_works_from_other_series() {
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
            work("a-1", "series-a", "2026-01-01T00:00:00Z"),
            work("b-1", "series-b", "2026-01-01T00:00:00Z"),
        ];

        write_series_pages(dist_dir.path(), &works, &series).unwrap();

        let html_a =
            fs::read_to_string(dist_dir.path().join("series/series-a/index.html")).unwrap();
        assert!(html_a.contains("a-1"));
        assert!(!html_a.contains("b-1"));

        let html_b =
            fs::read_to_string(dist_dir.path().join("series/series-b/index.html")).unwrap();
        assert!(html_b.contains("b-1"));
        assert!(!html_b.contains("a-1"));
    }

    #[test]
    fn write_series_pages_generates_a_page_per_series() {
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
}
