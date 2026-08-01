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
//!
//! 追記 (issue #9 時点): タグ別一覧ページ生成 (`dist/tags/<slug>/index.html`) は
//! [`build::tags`] として実装済み。#2 (Tag⇔Work 多対多)・#6・#11 は依然未マージのため、
//! 上記の「暫定モデル/差し替え想定」は変わらず有効。

pub mod build;
pub mod models;
pub mod rfc822;
pub mod xml_writer;
