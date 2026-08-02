//! `topcoat-render` — `topcoat` のテンプレートレンダリング・OG 画像生成を担うライブラリ crate。
//!
//! Tera によるトップページレンダリング ([`render_index`])・作品詳細ページレンダリング
//! ([`work_detail`]) と、`resvg`/`tiny-skia` による OG 画像生成
//! ([`og_image`] / [`raster`] / [`svg_template`]) を提供する。
//! `topcoat` 側の `xml_writer` (XML エスケープ処理) には依存しない設計とする
//! (SVG 用のエスケープ処理は [`svg_template`] 内に小さく独立実装している)。
//!
//! issue #30 (フロントエンド基盤) により、header/nav/footer/OGP メタを含む
//! 全ページ共通のベースレイアウトを [`layout`] (Askama) として追加した。
//! 既存の index/work_detail ページは Tera のままであり、[`layout`] はそれらを
//! 置き換えるものではなく、以降のフロントエンド系 issue がこのレイアウトを
//! 継承していくための土台として導入している。

use std::fmt;
use std::path::PathBuf;

use tera::{Context, Tera};

pub mod layout;
pub mod og_image;
pub mod raster;
pub mod svg_template;
pub mod work_detail;

pub use layout::BaseLayout;
pub use og_image::{generate_og_image, write_og_images, OgWork};
pub use raster::{rasterize_svg, RenderError as RasterError};
pub use svg_template::build_og_svg;

/// テンプレートのレンダリングに関するエラー。
#[derive(Debug)]
pub enum TemplateRenderError {
    /// Tera でのテンプレート読み込み・レンダリングに失敗した。
    Template(tera::Error),
}

impl fmt::Display for TemplateRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateRenderError::Template(err) => {
                write!(f, "テンプレートのレンダリングに失敗しました: {err}")
            }
        }
    }
}

impl std::error::Error for TemplateRenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TemplateRenderError::Template(err) => Some(err),
        }
    }
}

impl From<tera::Error> for TemplateRenderError {
    fn from(err: tera::Error) -> Self {
        TemplateRenderError::Template(err)
    }
}

/// `templates/` ディレクトリの中身をもとに Tera エンジンを構築する。
///
/// `CARGO_MANIFEST_DIR`（= `crates/topcoat-render`）基準でテンプレートディレクトリを
/// 解決するため、実行時のカレントディレクトリ（CWD）に依存しない。
fn build_engine() -> Result<Tera, TemplateRenderError> {
    let glob_pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("templates")
        .join("**")
        .join("*.tera");

    Tera::new(&glob_pattern.to_string_lossy()).map_err(TemplateRenderError::from)
}

/// トップページ (`index.html.tera`) をレンダリングし、HTML 文字列を返す。
pub fn render_index() -> Result<String, TemplateRenderError> {
    let engine = build_engine()?;
    let context = Context::new();
    engine
        .render("index.html.tera", &context)
        .map_err(TemplateRenderError::from)
}

/// `templates/**/*.tera` をロードした [`Tera`] インスタンスを構築する。
///
/// `CARGO_MANIFEST_DIR` (このクレートのルート) 基準でテンプレートディレクトリを解決するため、
/// 呼び出し側のカレントディレクトリに依存せず動作する。
pub fn build_tera() -> tera::Result<Tera> {
    let mut templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    templates_dir.push("templates");
    templates_dir.push("**");
    templates_dir.push("*.tera");

    let mut tera = Tera::new(
        templates_dir
            .to_str()
            .expect("CARGO_MANIFEST_DIR should be valid UTF-8"),
    )?;

    // テンプレートファイル名の拡張子が `.tera` (例: `work_detail.html.tera`) であり、
    // Tera のデフォルト自動エスケープ判定 (`.html` / `.htm` / `.xml` 終端) の対象外となるため、
    // `.tera` 終端のテンプレートを明示的に自動エスケープ対象にする。
    tera.autoescape_on(vec![".tera"]);

    Ok(tera)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_index_succeeds_and_contains_base_layout() {
        let html = render_index().expect("index.html.tera のレンダリングに失敗しました");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>topcoat</title>"));
    }
}
