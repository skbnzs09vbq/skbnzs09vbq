//! topcoat: 静的サイトジェネレータのビルドロジック。
//!
//! 現時点 (issue #15 着手時点) では、本 crate が前提とする以下の issue が
//! いずれも未着手・未マージである:
//!
//! - #3  Rust workspace 初期化と crate 構成決定
//! - #4  SQLite + Diesel ORM 導入
//! - #6  Work エンティティのテーブル定義と Diesel モデル実装
//! - #9  タグ別一覧ページ生成
//! - #10 シリーズ別一覧ページ生成
//! - #11 SSG ビルドエントリーポイント実装 (`topcoat build` コマンド骨格)
//!
//! そのため [`models`] は Diesel モデルではなく暫定的なプレーンな struct として
//! 定義している。上記 issue のマージ後は、フィールド名・型の互換性を保ったまま
//! Diesel モデルに差し替える想定。

pub mod build;
pub mod models;
pub mod rfc822;
pub mod xml_writer;
