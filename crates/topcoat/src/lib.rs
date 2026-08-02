//! topcoat: 静的サイトジェネレータのビルドロジック。
//!
//! 現時点では、本 crate が前提とする以下の issue が未着手・未マージである:
//!
//! - #6  Work エンティティのテーブル定義と Diesel モデル実装
//! - #9  タグ別一覧ページ生成
//! - #10 シリーズ別一覧ページ生成
//!
//! そのため [`models`] は Diesel モデルではなく暫定的なプレーンな struct として
//! 定義している。上記 issue のマージ後は、フィールド名・型の互換性を保ったまま
//! Diesel モデルに差し替える想定。同様に [`build::run`] が行う Work/Tag/Series/Version
//! の全件取得も、現時点では topcoat-db に実テーブルが無いため空データを返すプレースホルダ
//! 実装になっている。

pub mod build;
pub mod models;
pub mod rfc822;
pub mod xml_writer;
