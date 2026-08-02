//! 作品詳細ページ (`work_detail.html.tera`) のレンダリング。
//!
//! [`WorkDetailContext`] はテンプレートに渡すデータの形を表す。呼び出し側 (`topcoat`
//! クレートの `build::work_detail`) が [`crate::models::Work`][models-work] 相当の
//! データからこの構造体を組み立て、[`render_work_detail`] でレンダリングする。
//!
//! [models-work]: https://docs.rs/topcoat (crate 分割の都合上、直接の相互参照は行わない)

use serde::Serialize;
use tera::{Context, Tera};

/// タグ 1 件分のテンプレート向け表現。
#[derive(Debug, Clone, Serialize)]
pub struct TagRef {
    pub slug: String,
    pub name: String,
}

/// シリーズのテンプレート向け表現。
#[derive(Debug, Clone, Serialize)]
pub struct SeriesRef {
    pub slug: String,
    pub name: String,
}

/// 関連作品 1 件分のテンプレート向け表現。
#[derive(Debug, Clone, Serialize)]
pub struct RelatedWorkRef {
    pub slug: String,
    pub title: String,
}

/// `work_detail.html.tera` に渡すコンテキスト。
///
/// `tags` / `series` / `params` / `related_works` は #6 (Work エンティティ)・#7 (関連作品
/// 算出ロジック) が未マージの間は暫定データが渡される想定。
#[derive(Debug, Clone, Serialize)]
pub struct WorkDetailContext {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<TagRef>,
    pub series: Option<SeriesRef>,
    /// 作品の生成パラメータ。スキーマ未確定のため自由形式の JSON 値として扱う。
    pub params: serde_json::Value,
    pub related_works: Vec<RelatedWorkRef>,
}

const TEMPLATE_NAME: &str = "work_detail.html.tera";

/// `ctx` から作品詳細ページの HTML 文字列を描画する。
pub fn render_work_detail(tera: &Tera, ctx: &WorkDetailContext) -> tera::Result<String> {
    let context = Context::from_serialize(ctx)?;
    tera.render(TEMPLATE_NAME, &context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_tera;

    fn context() -> WorkDetailContext {
        WorkDetailContext {
            slug: "work-1".to_string(),
            title: "作品1".to_string(),
            description: "作品1の説明".to_string(),
            tags: vec![TagRef {
                slug: "shader".to_string(),
                name: "シェーダー".to_string(),
            }],
            series: Some(SeriesRef {
                slug: "series-1".to_string(),
                name: "シリーズ1".to_string(),
            }),
            params: serde_json::json!({ "seed": 42 }),
            related_works: vec![RelatedWorkRef {
                slug: "work-2".to_string(),
                title: "作品2".to_string(),
            }],
        }
    }

    #[test]
    fn render_includes_title_and_description() {
        let tera = build_tera().expect("template should load");
        let html = render_work_detail(&tera, &context()).expect("render should succeed");
        assert!(html.contains("作品1"));
        assert!(html.contains("作品1の説明"));
    }

    #[test]
    fn render_includes_tags_with_link() {
        let tera = build_tera().expect("template should load");
        let html = render_work_detail(&tera, &context()).expect("render should succeed");
        assert!(html.contains("/tags/shader/"));
        assert!(html.contains("シェーダー"));
    }

    #[test]
    fn render_includes_series_link_when_present() {
        let tera = build_tera().expect("template should load");
        let html = render_work_detail(&tera, &context()).expect("render should succeed");
        assert!(html.contains("/series/series-1/"));
        assert!(html.contains("シリーズ1"));
    }

    #[test]
    fn render_omits_series_section_when_absent() {
        let tera = build_tera().expect("template should load");
        let mut ctx = context();
        ctx.series = None;
        let html = render_work_detail(&tera, &ctx).expect("render should succeed");
        assert!(!html.contains("/series/"));
    }

    #[test]
    fn render_includes_related_works_links() {
        let tera = build_tera().expect("template should load");
        let html = render_work_detail(&tera, &context()).expect("render should succeed");
        assert!(html.contains("/works/work-2/"));
        assert!(html.contains("作品2"));
    }

    #[test]
    fn render_includes_canvas_placeholder_with_slug() {
        let tera = build_tera().expect("template should load");
        let html = render_work_detail(&tera, &context()).expect("render should succeed");
        assert!(html.contains(r#"id="work-canvas""#));
        assert!(html.contains(r#"data-slug="work-1""#));
    }

    #[test]
    fn render_includes_og_image_tag() {
        let tera = build_tera().expect("template should load");
        let html = render_work_detail(&tera, &context()).expect("render should succeed");
        assert!(html.contains("/works/work-1/og.png"));
    }
}
