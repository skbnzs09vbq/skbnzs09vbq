//! `topcoat-render` — `topcoat` のテンプレートレンダリング・OG 画像生成を担うライブラリ crate。
//!
//! 現時点では Tera によるテンプレートレンダリングのみを提供する。OG 画像生成等は
//! 後続 issue で実装する。

use std::fmt;
use std::path::PathBuf;

use tera::{Context, Tera};

/// テンプレートのレンダリングに関するエラー。
#[derive(Debug)]
pub enum RenderError {
    /// Tera でのテンプレート読み込み・レンダリングに失敗した。
    Template(tera::Error),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::Template(err) => {
                write!(f, "テンプレートのレンダリングに失敗しました: {err}")
            }
        }
    }
}

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RenderError::Template(err) => Some(err),
        }
    }
}

impl From<tera::Error> for RenderError {
    fn from(err: tera::Error) -> Self {
        RenderError::Template(err)
    }
}

/// `templates/` ディレクトリの中身をもとに Tera エンジンを構築する。
///
/// `CARGO_MANIFEST_DIR`（= `crates/topcoat-render`）基準でテンプレートディレクトリを
/// 解決するため、実行時のカレントディレクトリ（CWD）に依存しない。
fn build_engine() -> Result<Tera, RenderError> {
    let glob_pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("templates")
        .join("**")
        .join("*.tera");

    Tera::new(&glob_pattern.to_string_lossy()).map_err(RenderError::from)
}

/// トップページ (`index.html.tera`) をレンダリングし、HTML 文字列を返す。
pub fn render_index() -> Result<String, RenderError> {
    let engine = build_engine()?;
    let context = Context::new();
    engine
        .render("index.html.tera", &context)
        .map_err(RenderError::from)
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
