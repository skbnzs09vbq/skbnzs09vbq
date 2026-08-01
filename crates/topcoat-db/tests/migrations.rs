//! `topcoat-db` の疎通確認テスト。
//!
//! 「SQLite への接続確立 → 埋め込みマイグレーションの適用」が正常終了することのみを
//! 確認する（本 issue 時点では実テーブル定義が存在しないため、内容の検証は行わない）。
//!
//! `cargo test` はデフォルトで同一バイナリ内のテストを並行実行するため、各テストは
//! `tempfile` で払い出した一意な一時パスへ接続する（固定パスを共有すると SQLite の
//! 書き込みロック競合で間欠的に失敗するため）。

use tempfile::NamedTempFile;

#[test]
fn establish_connection_and_run_migrations_succeeds() {
    let db_file = NamedTempFile::new().expect("一時 DB ファイルの作成に失敗しました");

    let mut connection = topcoat_db::establish_connection_at(db_file.path())
        .expect("SQLite への接続確立に失敗しました");

    topcoat_db::run_migrations(&mut connection).expect("マイグレーションの適用に失敗しました");
}

#[test]
fn run_migrations_is_idempotent() {
    let db_file = NamedTempFile::new().expect("一時 DB ファイルの作成に失敗しました");

    let mut connection = topcoat_db::establish_connection_at(db_file.path())
        .expect("SQLite への接続確立に失敗しました");

    topcoat_db::run_migrations(&mut connection).expect("1回目のマイグレーション適用に失敗しました");
    topcoat_db::run_migrations(&mut connection).expect("2回目のマイグレーション適用に失敗しました");
}
