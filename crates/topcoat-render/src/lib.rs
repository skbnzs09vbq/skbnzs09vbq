//! `topcoat-render` — `topcoat` のテンプレートレンダリング・OG 画像生成を担うライブラリ crate。
//!
//! 現時点では作品詳細ページ (`work_detail`) のテンプレートレンダリングのみを実装している。
//! OG 画像生成は別 issue (#13) の範囲。

pub mod work_detail;

use std::path::PathBuf;

use tera::Tera;

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
