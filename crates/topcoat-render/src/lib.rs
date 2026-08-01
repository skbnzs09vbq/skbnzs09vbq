//! `topcoat-render` — `topcoat` のテンプレートレンダリング・OG 画像生成を担うライブラリ crate。
//!
//! Tera によるテンプレートレンダリング ([`render_index`]) と、`resvg`/`tiny-skia` による
//! OG 画像生成 ([`og_image`] / [`raster`] / [`svg_template`]) の 2 系統の機能を提供する。
//! `topcoat` 側の `xml_writer` (XML エスケープ処理) には依存しない設計とする
//! (SVG 用のエスケープ処理は [`svg_template`] 内に小さく独立実装している)。

use std::fmt;
use std::path::PathBuf;

use tera::{Context, Tera};

pub mod og_image;
pub mod raster;
pub mod svg_template;

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
