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

use std::path::PathBuf;

use tera::Tera;

pub use og_image::{generate_og_image, write_og_images, OgWork};
pub use raster::{rasterize_svg, RenderError};
pub use svg_template::build_og_svg;

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
