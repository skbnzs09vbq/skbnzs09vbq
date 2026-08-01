//! `topcoat-render` — `topcoat` のテンプレートレンダリング・OG 画像生成を担うライブラリ crate。
//!
//! `topcoat` 側の `xml_writer` (XML エスケープ処理) には依存しない設計とする
//! (SVG 用のエスケープ処理は [`svg_template`] 内に小さく独立実装している)。

pub mod og_image;
pub mod raster;
pub mod svg_template;

pub use og_image::{generate_og_image, write_og_images, OgWork};
pub use raster::{rasterize_svg, RenderError};
pub use svg_template::build_og_svg;
