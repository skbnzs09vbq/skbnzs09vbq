//! 統合テスト間で共通して使う DB セットアップ処理。

use diesel::sqlite::SqliteConnection;
use tempfile::NamedTempFile;

/// 一時 DB ファイルへの接続を確立し、マイグレーションを適用する。
///
/// `cargo test` はデフォルトで同一バイナリ内のテストを並行実行するため、各テストは
/// `tempfile` で払い出した一意な一時パスへ接続する（固定パスを共有すると SQLite の
/// 書き込みロック競合で間欠的に失敗するため）。
///
/// 返り値の [`NamedTempFile`] は drop されると一時ファイルが削除されるため、
/// 呼び出し側で（`_` プレフィックス変数などに束縛して）テスト終了まで保持しておく必要がある。
pub fn setup_connection() -> (NamedTempFile, SqliteConnection) {
    let db_file = NamedTempFile::new().expect("一時 DB ファイルの作成に失敗しました");

    let mut connection = topcoat_db::establish_connection_at(db_file.path())
        .expect("SQLite への接続確立に失敗しました");
    topcoat_db::run_migrations(&mut connection).expect("マイグレーションの適用に失敗しました");

    (db_file, connection)
}
