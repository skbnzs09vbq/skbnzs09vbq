//! `topcoat-render` — `topcoat` のテンプレートレンダリング・OG 画像生成を担うライブラリ crate。
//!
//! 作品詳細ページ (`work_detail`) のテンプレートレンダリングと、OG 画像生成
//! (`og_image` / `raster` / `svg_template`) の両方を実装している。
//! `topcoat` 側の `xml_writer` (XML エスケープ処理) には依存しない設計とする
//! (SVG 用のエスケープ処理は [`svg_template`] 内に小さく独立実装している)。

pub mod og_image;
pub mod raster;
pub mod svg_template;
pub mod work_detail;

use tera::Tera;

pub use og_image::{generate_og_image, write_og_images, OgWork};
pub use raster::{rasterize_svg, RenderError};
pub use svg_template::build_og_svg;

const WORK_DETAIL_TEMPLATE: &str = include_str!("../templates/work_detail.html.tera");
const WORK_DETAIL_TEMPLATE_NAME: &str = "work_detail.html.tera";

/// [`WORK_DETAIL_TEMPLATE`] を登録した [`Tera`] インスタンスを構築する。
///
/// テンプレートはビルド時に [`include_str!`] でバイナリへ埋め込む。
/// `crates/topcoat/src/build/series.rs` の `series_tera()` と同様の方針
/// (ディレクトリ glob 方式 (`Tera::new("templates/**/*")`) を採らない) で、
/// ビルドしたマシン・ディレクトリと異なる環境でバイナリを実行しても、実行時
/// カレントディレクトリや `CARGO_MANIFEST_DIR` の絶対パスに依存せず、テンプレート
/// ファイルの配置漏れによる実行時エラーを避けるため。
pub fn build_tera() -> tera::Result<Tera> {
    let mut tera = Tera::default();

    // テンプレートファイル名の拡張子が `.tera` (例: `work_detail.html.tera`) であり、
    // Tera のデフォルト自動エスケープ判定 (`.html` / `.htm` / `.xml` 終端) の対象外となるため、
    // `.tera` 終端のテンプレートを明示的に自動エスケープ対象にする。
    tera.autoescape_on(vec![".tera"]);

    tera.add_raw_template(WORK_DETAIL_TEMPLATE_NAME, WORK_DETAIL_TEMPLATE)?;

    Ok(tera)
}
