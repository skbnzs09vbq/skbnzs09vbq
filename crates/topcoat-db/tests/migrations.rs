//! `topcoat-db` の疎通確認テスト。
//!
//! 「SQLite への接続確立 → 埋め込みマイグレーションの適用」が正常終了すること、および
//! 複数回の適用が冪等であることを確認する（テーブル内容自体の検証は `tests/models.rs` で行う）。
//!
//! `cargo test` はデフォルトで同一バイナリ内のテストを並行実行するため、各テストは
//! `tempfile` で払い出した一意な一時パスへ接続する（固定パスを共有すると SQLite の
//! 書き込みロック競合で間欠的に失敗するため）。

mod common;

#[test]
fn establish_connection_and_run_migrations_succeeds() {
    let _ = common::setup_connection();
}

#[test]
fn run_migrations_is_idempotent() {
    let (_db_file, mut connection) = common::setup_connection();

    topcoat_db::run_migrations(&mut connection).expect("2回目のマイグレーション適用に失敗しました");
}
