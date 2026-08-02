//! `dist/tags/<slug>/index.html` の生成（タグ別一覧ページ）。
//!
//! 該当タグが付与された全 Work のサムネイル・タイトル一覧を Tera テンプレートに
//! 埋め込んで HTML を生成する。
//!
//! 本来 Tera テンプレートエンジンの導入・共通レイアウトの用意は `topcoat-render` crate
//! （#11 で本実装予定）が担う想定だが、issue #9 着手時点で #2・#6・#11 がいずれも
//! 未マージのため、本 crate 内で完結する形で最小限のテンプレートを直接埋め込んで
//! 実装している。#11 マージ後は共通レイアウト・`topcoat-render` 経由のレンダリングへの
//! 差し替えを検討すること。

use std::fs;
use std::io;
use std::path::Path;

use serde::Serialize;
use tera::{Context, Tera};

/// タグ一覧ページテンプレートの Tera 上の名前。
///
/// `.html` サフィックスにより Tera の自動エスケープが有効になる。
const TEMPLATE_NAME: &str = "tags/index.html";

/// テンプレート本体をビルド時にバイナリへ埋め込む
/// (`crates/topcoat-db` の `embed_migrations!` と同様、実行時 CWD に依存させないため)。
const TEMPLATE_SOURCE: &str = include_str!("../../templates/tags/index.html.tera");

/// タグ一覧ページに掲載する Work 1件分の情報。
///
/// [`crate::models::Work`] からのマッピングは呼び出し側 (`build::mod`) が行う。
#[derive(Debug, Clone, Serialize)]
pub struct TagPageWork {
    pub slug: String,
    pub title: String,
    /// サムネイル画像の URL・パス。`None` の場合はテンプレート側で `<img>` を出力しない。
    pub thumbnail: Option<String>,
}

/// タグ一覧ページ 1件分の生成に必要な情報。
#[derive(Debug, Clone)]
pub struct TagPageEntry {
    pub slug: String,
    pub name: String,
    /// このタグが付与された Work 一覧 (呼び出し側でフィルタ済みであること)。
    /// 並び順は呼び出し側から渡された順をそのまま保持する。
    pub works: Vec<TagPageWork>,
}

#[derive(Debug, Serialize)]
struct TagContext<'a> {
    slug: &'a str,
    name: &'a str,
}

/// タグ一覧ページテンプレートを読み込み済みの [`Tera`] インスタンスを構築する。
///
/// テンプレートは静的なため、呼び出し側で一度だけ構築して使い回すこと。
pub fn build_tera() -> Result<Tera, tera::Error> {
    let mut tera = Tera::default();
    tera.add_raw_template(TEMPLATE_NAME, TEMPLATE_SOURCE)?;
    Ok(tera)
}

/// 1タグ分の `index.html` を文字列として生成する。
///
/// この関数自体はファイル I/O を行わない純粋関数。
/// `tera` は呼び出し側で一度だけ構築したインスタンスを渡すこと
/// （テンプレートは静的なため、呼び出しごとに再構築・再パースする必要はない）。
pub fn render_tag_page(tera: &Tera, entry: &TagPageEntry) -> Result<String, tera::Error> {
    let mut context = Context::new();
    context.insert(
        "tag",
        &TagContext {
            slug: &entry.slug,
            name: &entry.name,
        },
    );
    context.insert("works", &entry.works);

    tera.render(TEMPLATE_NAME, &context)
}

/// `entries` の各タグについて `dist_dir/tags/<slug>/index.html` を書き出す。
pub fn write_tag_pages(dist_dir: &Path, entries: &[TagPageEntry]) -> io::Result<()> {
    let tera = build_tera().map_err(|err| io::Error::other(err.to_string()))?;

    for entry in entries {
        let html =
            render_tag_page(&tera, entry).map_err(|err| io::Error::other(err.to_string()))?;

        let out_dir = dist_dir.join("tags").join(&entry.slug);
        fs::create_dir_all(&out_dir)?;
        fs::write(out_dir.join("index.html"), html)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(slug: &str, name: &str, works: Vec<TagPageWork>) -> TagPageEntry {
        TagPageEntry {
            slug: slug.to_string(),
            name: name.to_string(),
            works,
        }
    }

    #[test]
    fn renders_tag_name_and_no_items_when_no_works() {
        let tera = build_tera().unwrap();
        let html = render_tag_page(&tera, &entry("illustration", "イラスト", vec![])).unwrap();
        assert!(html.contains("<h1>イラスト</h1>"));
        assert!(html.contains("<title>イラスト の作品一覧</title>"));
        assert!(!html.contains("work-item"));
    }

    #[test]
    fn renders_work_title_and_link() {
        let tera = build_tera().unwrap();
        let html = render_tag_page(
            &tera,
            &entry(
                "illustration",
                "イラスト",
                vec![TagPageWork {
                    slug: "work-1".to_string(),
                    title: "作品1".to_string(),
                    thumbnail: None,
                }],
            ),
        )
        .unwrap();
        assert!(html.contains(r#"<a class="work-link" href="/works/work-1/">作品1</a>"#));
    }

    #[test]
    fn renders_thumbnail_img_when_present() {
        let tera = build_tera().unwrap();
        let html = render_tag_page(
            &tera,
            &entry(
                "illustration",
                "イラスト",
                vec![TagPageWork {
                    slug: "work-1".to_string(),
                    title: "作品1".to_string(),
                    thumbnail: Some("/thumbs/work-1.png".to_string()),
                }],
            ),
        )
        .unwrap();
        // Tera の自動エスケープにより `/` は `&#x2F;` にエンティティ化される
        // （ブラウザは属性値中でこれを `/` として正しくデコードするため実害はない）。
        assert!(html.contains(
            r#"<img class="work-thumbnail" src="&#x2F;thumbs&#x2F;work-1.png" alt="作品1">"#
        ));
    }

    #[test]
    fn omits_thumbnail_img_when_absent() {
        let tera = build_tera().unwrap();
        let html = render_tag_page(
            &tera,
            &entry(
                "illustration",
                "イラスト",
                vec![TagPageWork {
                    slug: "work-1".to_string(),
                    title: "作品1".to_string(),
                    thumbnail: None,
                }],
            ),
        )
        .unwrap();
        assert!(!html.contains("work-thumbnail"));
    }

    #[test]
    fn includes_all_works_preserving_input_order() {
        let tera = build_tera().unwrap();
        let html = render_tag_page(
            &tera,
            &entry(
                "illustration",
                "イラスト",
                vec![
                    TagPageWork {
                        slug: "work-2".to_string(),
                        title: "2番目".to_string(),
                        thumbnail: None,
                    },
                    TagPageWork {
                        slug: "work-1".to_string(),
                        title: "1番目".to_string(),
                        thumbnail: None,
                    },
                ],
            ),
        )
        .unwrap();
        let pos_2 = html.find("works/work-2/").unwrap();
        let pos_1 = html.find("works/work-1/").unwrap();
        assert!(pos_2 < pos_1, "呼び出し側が渡した順が保持されること");
    }

    #[test]
    fn escapes_special_characters_in_title_and_name() {
        let tera = build_tera().unwrap();
        let html = render_tag_page(
            &tera,
            &entry(
                "special",
                "<script>タグ</script>",
                vec![TagPageWork {
                    slug: "work-1".to_string(),
                    title: "<b>強調</b> & \"quote\"".to_string(),
                    thumbnail: None,
                }],
            ),
        )
        .unwrap();
        assert!(!html.contains("<script>タグ</script>"));
        assert!(!html.contains("<b>強調</b>"));
        // Tera の自動エスケープは `/` も `&#x2F;` にエスケープする。
        assert!(html.contains("&lt;script&gt;タグ&lt;&#x2F;script&gt;"));
        assert!(html.contains("&lt;b&gt;強調&lt;&#x2F;b&gt; &amp; &quot;quote&quot;"));
    }

    #[test]
    fn escapes_special_characters_in_thumbnail_and_slug() {
        let tera = build_tera().unwrap();
        let html = render_tag_page(
            &tera,
            &entry(
                "illustration",
                "イラスト",
                vec![TagPageWork {
                    slug: "foo\"><script>alert(1)</script>".to_string(),
                    title: "作品1".to_string(),
                    thumbnail: Some("x\" onerror=\"alert(1)".to_string()),
                }],
            ),
        )
        .unwrap();
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(!html.contains(r#"" onerror="alert(1)"#));
        assert!(html.contains("&quot;&gt;&lt;script&gt;alert(1)&lt;&#x2F;script&gt;"));
        assert!(html.contains("x&quot; onerror=&quot;alert(1)"));
    }

    #[test]
    fn write_tag_pages_writes_one_file_per_tag() {
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
            entry("manga", "漫画", vec![]),
        ];

        write_tag_pages(dir.path(), &entries).unwrap();

        let illustration_html =
            fs::read_to_string(dir.path().join("tags/illustration/index.html")).unwrap();
        assert!(illustration_html.contains("作品1"));

        let manga_html = fs::read_to_string(dir.path().join("tags/manga/index.html")).unwrap();
        assert!(manga_html.contains("<h1>漫画</h1>"));
    }
}
