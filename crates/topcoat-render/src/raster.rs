//! SVG 文字列を PNG バイト列にラスタライズする。
//!
//! `usvg` で SVG をパースし、埋め込みフォント (`assets/ArchivoBlack-Regular.ttf`, OFL ライセンス)
//! のみをロードした `fontdb::Database` を使ってテキストを解決する。システムフォントには
//! 依存しない (`fontdb::Database::load_system_fonts` は呼ばない) ため、開発機・CI・将来のビルド
//! 環境が変わってもレンダリング結果は変わらない。
//!
//! `fontdb` は直接の依存としては追加せず、`usvg::fontdb` (再エクスポート) を使う。
//! `usvg` が内部で要求する `fontdb` のバージョンと、直接依存として追加した `fontdb` の
//! バージョンがずれると `usvg::Options::fontdb` に渡す型が一致しなくなるため。

use std::fmt;
use std::sync::{Arc, OnceLock};

use usvg::fontdb;

/// リポジトリに同梱している OG 画像用フォント (OFL ライセンス)。
///
/// システムフォントに依存するとビルド環境差でレンダリング結果が変わるため、
/// バイナリに埋め込み `fontdb::Database` へ明示的にロードする。
const EMBEDDED_FONT: &[u8] = include_bytes!("../assets/ArchivoBlack-Regular.ttf");

/// [`rasterize_svg`] が失敗した場合のエラー。
#[derive(Debug)]
pub enum RenderError {
    /// SVG のパースに失敗した。
    Parse(String),
    /// `tiny_skia::Pixmap` の生成に失敗した (幅・高さが不正等)。
    Pixmap,
    /// PNG へのエンコードに失敗した。
    Encode(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::Parse(message) => write!(f, "failed to parse SVG: {message}"),
            RenderError::Pixmap => write!(f, "failed to create pixmap"),
            RenderError::Encode(message) => write!(f, "failed to encode PNG: {message}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// 埋め込みフォントをロードした [`fontdb::Database`] を構築する。
///
/// フォント自体は不変な定数のため、[`font_database`] からプロセス内で1度だけ呼び出され
/// キャッシュされる。
fn build_font_database() -> Arc<fontdb::Database> {
    let mut db = fontdb::Database::new();
    db.load_font_data(EMBEDDED_FONT.to_vec());
    Arc::new(db)
}

/// 埋め込みフォントをロードした [`fontdb::Database`] を取得する。
///
/// 初回呼び出し時にのみ構築し、以降は同じ `Arc` を使い回す。フォント自体は不変な定数であり、
/// `rasterize_svg` の呼び出しごとに埋め込みフォントのコピー・パースが発生するのを避ける。
fn font_database() -> Arc<fontdb::Database> {
    static FONT_DATABASE: OnceLock<Arc<fontdb::Database>> = OnceLock::new();
    FONT_DATABASE.get_or_init(build_font_database).clone()
}

/// SVG 文字列を PNG バイト列にラスタライズする。
///
/// SVG のルート要素の `width`/`height` (または `viewBox`) がそのまま出力 PNG の解像度になる。
pub fn rasterize_svg(svg: &str) -> Result<Vec<u8>, RenderError> {
    let options = usvg::Options {
        fontdb: font_database(),
        ..usvg::Options::default()
    };

    let tree =
        usvg::Tree::from_str(svg, &options).map_err(|err| RenderError::Parse(err.to_string()))?;

    let size = tree.size();
    let width = size.width().round() as u32;
    let height = size.height().round() as u32;

    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or(RenderError::Pixmap)?;

    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|err| RenderError::Encode(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg_template::build_og_svg;

    const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    #[test]
    fn rasterize_svg_produces_valid_png() {
        let svg = build_og_svg("Hello, OG!", "hello-og");
        let png = rasterize_svg(&svg).expect("rasterize should succeed");

        assert!(!png.is_empty());
        assert!(png.starts_with(&PNG_SIGNATURE));
    }

    #[test]
    fn rasterize_svg_rejects_invalid_svg() {
        let result = rasterize_svg("not an svg document");
        assert!(result.is_err());
    }
}
